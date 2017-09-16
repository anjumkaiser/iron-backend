use iron::prelude::*;
use iron::status;
use router::*;
use bodyparser;

use persistent::{Read, Write};
use hyper::header::*;
use hyper::mime::*;
use dal;
use std::ops::Deref;
use r2d2_postgres;
use time::{Timespec, Tm, at_utc};
use chrono::prelude::*;
use serde_json;
use std;
use std::str;

use uuid;
use bcrypt;

use jsonwebtoken::{encode, Header};

use configmisc;

use slog;

use server::loggerenclave::LoggerEnclave;



#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}


pub fn index_handler(req: &mut Request) -> IronResult<Response> {

    let logger: slog::Logger = get_logger!(req);

    info!(logger, "inside handler");

    let respdata = "Hello";
    Ok(Response::with((status::Ok, respdata)))
}


pub fn index_handler2(_: &mut Request) -> IronResult<Response> {
    let respdata = r#"{"key","value"}"#.as_bytes();
    let mut resp = Response::with((status::Ok, respdata));
    resp.headers = Headers::new();
    resp.headers.set(ContentType(Mime(
        TopLevel::Application,
        SubLevel::Json,
        vec![(Attr::Charset, Value::Utf8)],
    )));
    Ok(resp)
}

pub fn index_handler3(req: &mut Request) -> IronResult<Response> {

    let mut resp = Response::with((status::NotFound));

    //let logger = req.get_logger();

    //info!(logger, "Request recvd : {:?}", req);

    println!("Url: {:?}", req.url.path());
    if let Some(params) = req.extensions.get::<Router>() {
        println!("Params {:?}", params["name"]);

        if let Some(name_param) = params.find("name") {
            println!("Found param name : {}", name_param);
            resp = Response::with((status::Ok, "text data"));
            resp.headers = Headers::new();
            resp.headers.set(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![]),
            ));
        }
    }

    Ok(resp)
}

pub fn get_db_time(req: &mut Request) -> IronResult<Response> {

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DbData {
        pub Id: i32,
        pub Name: String,
        pub Timestamp: i64,
    }


    println!("in get_db_time");
    let mut resp = Response::with((status::NotFound));

    match req.get::<Write<dal::DalPostgresPool>>() {
        Ok(arcpool) => {
            match arcpool.lock() {
                Ok(x) => {
                    let pool = x.deref();
                    if let Ok(conn) = pool.rw_pool.get() {
                        if let Ok(stmt) = conn.prepare("SELECT 1 as id, 'someone' as name, now() as timestamp") {
                            if let Ok(rows) = stmt.query(&[]) {
                                for row in rows.iter() {
                                    let _id: i32 = row.get("id");
                                    let _name: String = row.get("name");
                                    let _timestamp: Timespec = row.get("timestamp");
                                    let utc_tm: Tm = at_utc(_timestamp);
                                    let local_tm: Tm = utc_tm.to_local();
                                    println!(
                                        "row [{}, {}, utc {}, local {}] ",
                                        _id,
                                        _name,
                                        utc_tm.asctime(),
                                        local_tm.asctime()
                                    );

                                    let data: DbData = DbData {
                                        Id: _id,
                                        Name: _name,
                                        Timestamp: _timestamp.sec,
                                    };

                                    match serde_json::to_string(&data) {
                                        Ok(json_resp) => {
                                            resp = Response::with((status::Ok, json_resp));
                                            resp.headers.set(ContentType(
                                                Mime(TopLevel::Application, SubLevel::Json, vec![]),
                                            ));
                                        }
                                        _ => {}
                                    }

                                    break; // we only need first element
                                }
                            } else {
                                println!("unable to execute query");
                            }
                        } else {
                            println!("unable to prepare statement");
                        }
                    } else {
                        println!("unable to get connection from pool");
                    }
                }
                Err(e) => {
                    println!("Error {:?}", e);
                }
            }
        }
        Err(e) => {
            println!(" Error {:?}", e);
        }
    }

    Ok(resp)
}



pub fn authenticate(req: &mut Request) -> IronResult<Response> {


    println!("in authenticate");


    let mut resp = Response::with((status::NotFound));

    //let ref rhead = req.headers;
    //println!("rhead {}", rhead);
    //let ref rbody = req.body;
    //println!("rbody {}", rbody);

    let mut resp_content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));

    if req.headers.has::<ContentType>() {
        if let Some(ctype) = req.headers.get_raw("content-type") {
            if let Ok(strx) = str::from_utf8(&ctype[0]) {
                println!("content type received is {}", strx);
                if strx == "application/json" {
                    resp_content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));
                } else if strx == "application/cbor" {
                    resp_content_type = ContentType(Mime(
                        TopLevel::Application,
                        SubLevel::Ext("cbor".to_string()),
                        vec![],
                    ));
                } else if strx == "application/msgpack" {
                    resp_content_type = ContentType(Mime(
                        TopLevel::Application,
                        SubLevel::Ext("msgpack".to_string()),
                        vec![],
                    ));
                } else if strx == "applcaiton/protobuf" {
                    resp_content_type = ContentType(Mime(
                        TopLevel::Application,
                        SubLevel::Ext("protobuf".to_string()),
                        vec![],
                    ));
                } else {
                    // json
                }
            }
        } else {
            resp_content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));
        }
    }

    let rbody = req.get::<bodyparser::Json>();
    println!("rbody {:?}", rbody);

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct AuthUser {
        pub username: String,
        pub password: String,
    };

    if let Ok(Some(authuser)) = req.get::<bodyparser::Struct<AuthUser>>() {

        println!("authuser = {:?}", authuser);
        //let query = format!("select * from user where userid={}", authuser.username);
        let query = "select * from customer_local_auth where local_id=$1";
        println!("query [{}]", query);

        match req.get::<Write<dal::DalPostgresPool>>() {
            Ok(arcpool) => {
                match arcpool.lock() {
                    Ok(x) => {
                        let pool = x.deref();
                        if let Ok(conn) = pool.rw_pool.get() {
                            match conn.prepare(&query) {
                                Ok(stmt) => {
                                    if let Ok(rows) = stmt.query(&[&"admin".to_string()]) {
                                        if rows.is_empty() {
                                            println!("empty rows");
                                        } else {

                                            for row in rows.iter() {

                                                #[derive(Debug, Serialize, Deserialize)]
                                                struct CustomerLocalAuth {
                                                    pub customer_id_uuid: uuid::Uuid,
                                                    pub password_hash: String,
                                                }

                                                let c: CustomerLocalAuth = CustomerLocalAuth {
                                                    customer_id_uuid: row.get("customer_id_uuid"),
                                                    password_hash: row.get("password_hash"),
                                                };
                                                println!("c [{:?}]", c);
                                                if let Ok(res) = bcrypt::verify(&authuser.password, &c.password_hash) {
                                                    println!("res [{:?}]", res);
                                                    if res == true {
                                                        resp = Response::with((status::Ok));
                                                    }
                                                }

                                                break; // we only need first element
                                            }
                                        }
                                    } else {
                                        println!("unable to execute query");
                                    }
                                }
                                Err(e) => {
                                    println!("unable to prepare statement e {:?}", e);
                                }
                            }
                        } else {
                            println!("unable to get connection from pool");
                        }
                    }
                    Err(e) => {
                        println!("Error {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!(" Error {:?}", e);
            }
        }
    }

    //let ref rbody = req.body;
    //println!("line #{}", rbody);

    resp.headers.set(resp_content_type);

    Ok(resp)
}



pub fn backoffice_authenticate(req: &mut Request) -> IronResult<Response> {


    println!("in backoffice_authenticate");

    #[derive(Serialize, Deserialize, Debug)]
    struct PrivateClaims {
        user_id: uuid::Uuid,
        user_name: String,
        role_id: uuid::Uuid,
        role_name: String,
    };

    #[derive(Serialize, Deserialize, Debug)]
    struct Claims {
        sub: String, // subject
        iss: String, // issuer
        aud: Vec<String>, // audience
        exp: i64, // expiration time
        nbf: i64, // use not before
        iat: i64, // issued at time
        jti: String, // jwt id - case sensitive and unique among servers
        pvt: PrivateClaims,
    };


    let mut resp = Response::with((status::NotFound));

    //let ref rhead = req.headers;
    //println!("rhead {}", rhead);
    //let ref rbody = req.body;
    //println!("rbody {}", rbody);

    let mut resp_content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));

    if req.headers.has::<ContentType>() {
        if let Some(ctype) = req.headers.get_raw("content-type") {
            if let Ok(strx) = str::from_utf8(&ctype[0]) {
                println!("content type received is {}", strx);
                if strx == "application/json" {
                    resp_content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));
                } else if strx == "application/cbor" {
                    resp_content_type = ContentType(Mime(
                        TopLevel::Application,
                        SubLevel::Ext("cbor".to_string()),
                        vec![],
                    ));
                } else if strx == "application/msgpack" {
                    resp_content_type = ContentType(Mime(
                        TopLevel::Application,
                        SubLevel::Ext("msgpack".to_string()),
                        vec![],
                    ));
                } else if strx == "applcaiton/protobuf" {
                    resp_content_type = ContentType(Mime(
                        TopLevel::Application,
                        SubLevel::Ext("protobuf".to_string()),
                        vec![],
                    ));
                } else {
                    // json
                }
            }
        } else {
            resp_content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));
        }
    }

    let resp_headers = resp_content_type.clone();
    resp.headers.set(resp_content_type);


    let rbody = req.get::<bodyparser::Json>();
    println!("rbody {:?}", rbody);

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct AuthUser {
        pub username: String,
        pub password: String,
    };

    /*
    let authuser = match req.get::<bodyparser::Struct<AuthUser>>() {
        Ok(x) => {
            match x {
                Some(y) => y,
                None => {
                    return Ok(resp);
                }
            }
        }
        Err(e) => {
            return Ok(resp);
        }
    };
    */

    let authuser;

    if let Ok(Some(x)) = req.get::<bodyparser::Struct<AuthUser>>() {
        authuser = x;
    } else {
        return Ok(resp);
    }


    println!("authuser = {:?}", authuser);
    //let query = format!("select * from user where userid={}", authuser.username);
    let mut query: String = "SELECT ".to_string();
    query += "u.id as user_id, u.name as user_name, p.password as user_password, r.id as role_id, r.name as role_name";
    query += " FROM backoffice_user u, backoffice_user_password p, backoffice_user_role r";
    query += " WHERE p.user_id = u.id";
    query += " AND r.id = u.role_id";
    query += " AND u.name=$1";
    query += " order by p.date";
    println!("query [{}]", query);

    let cfgmisc;

    match req.get::<Read<configmisc::ConfigMisc>>() {
        Ok(dcfgmisc) => {
            cfgmisc = dcfgmisc.clone();
        }
        Err(_) => {
            return Ok(resp);
        }
    }


    let arcpool;
    let locked_pool;
    let pool;
    let conn;

    arcpool = match req.get::<Write<dal::DalPostgresPool>>() {
        Ok(x) => x,
        Err(e) => {
            println!("{:?}", e);
            return Ok(resp);
        }
    };

    locked_pool = match arcpool.lock() {
        Ok(x) => x,
        Err(e) => {
            println!("{:?}", e);
            return Ok(resp);
        }
    };


    pool = locked_pool.deref();

    conn = match pool.rw_pool.get() {
        Ok(x) => x,
        Err(e) => {
            println!("{:?}", e);
            return Ok(resp);
        }
    };


    let stmt = match conn.prepare(&query) {
        Ok(x) => x,
        Err(e) => {
            println!("{:?}", e);
            return Ok(resp);
        }
    };


    let rows = match stmt.query(&[&authuser.username]) {
        Ok(x) => x,
        Err(e) => {
            println!("{:?}", e);
            return Ok(resp);
        }
    };


    if rows.is_empty() {
        println!("empty rows");
        return Ok(resp);
    }

    for row in rows.iter() {

        #[derive(Debug, Serialize, Deserialize)]
        struct BackOfficeUserAuth {
            pub user_id: uuid::Uuid,
            pub user_name: String,
            pub role_id: uuid::Uuid,
            pub role_name: String,
            pub password_hash: String,
        }

        let c: BackOfficeUserAuth = BackOfficeUserAuth {
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            password_hash: row.get("user_password"),
            role_id: row.get("role_id"),
            role_name: row.get("role_name"),
        };
        println!("c [{:?}]", c);



        let res = match bcrypt::verify(&authuser.password, &c.password_hash) {
            Ok(x) => x,
            Err(e) => {
                return Ok(resp);
            }
        };
        println!("res [{:?}]", res);

        if true != res {
            break;
        }

        let mut jwt_issuer = String::new();
        {
            use url;
            let _url: url::Url = req.url.clone().into();
            jwt_issuer += _url.scheme();

            jwt_issuer += "://";
            if let Some(x) = _url.host_str() {
                println!("url host_str [{}]", x);
                jwt_issuer += x;
            }
        }

        let current_time = Utc::now();
        let jwt_issue_timestamp: i64 = current_time.timestamp();
        let jwt_exp_timestamp = jwt_issue_timestamp + (3600 * 1);

        /*
        let jissuetime = Utc.timestamp(jwt_issue_timestamp, 0);
        let jexptime = Utc.timestamp(jwt_exp_timestamp, 0);
        println!("jwt_iss_time {},  jwet_exp_time {}", jissuetime, jexptime);
        */

        //let remote_addr = .clone();

        let mut jwt_aud = Vec::new();
        jwt_aud.push(c.user_id.simple().to_string());
        jwt_aud.push(c.user_id.to_string());
        jwt_aud.push(c.user_name.clone());
        jwt_aud.push(req.remote_addr.to_string());


        println!("jwt_aud {:?}", jwt_aud);


        let private_claims = PrivateClaims {
            user_id: c.user_id,
            user_name: c.user_name,
            role_id: c.role_id,
            role_name: c.role_name,
        };

        let my_claims = Claims {
            sub: c.user_id.simple().to_string(), // populating uuid as simple format (hypen-less format) string
            iss: jwt_issuer,
            aud: jwt_aud,
            iat: jwt_issue_timestamp,
            exp: jwt_exp_timestamp,
            jti: uuid::Uuid::new_v4().simple().to_string(),
            nbf: jwt_issue_timestamp,
            pvt: private_claims,
        };

        let jwt_token: String;

        if let Ok(jtoken) = encode(
            &Header::default(),
            &my_claims,
            &cfgmisc.jwt_secret.clone().into_bytes(),
        )
        {
            jwt_token = jtoken;
        } else {
            return Ok(resp);
        }

        let resp_data: ResponseData<String> = ResponseData {
            success: true,
            data: jwt_token,
            message: "".to_owned(),
        };

        if let Ok(respjson) = serde_json::to_string(&resp_data) {
            resp = Response::with((status::Ok, respjson));
        }

        break; // we only need first element
    }

    resp.headers.set(resp_headers);
    Ok(resp)
}
