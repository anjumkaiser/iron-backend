use iron::prelude::*;
use iron::status;
use router::*;
use bodyparser;

use persistent::Write;
use hyper::header::*;
use hyper::mime::*;
use dal;
use std::ops::Deref;
use r2d2_postgres;
use time::{Timespec, Tm, at_utc};
use serde_json;
use std;
use std::str;

use uuid;
use bcrypt;


pub fn index_handler(_: &mut Request) -> IronResult<Response> {
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

    println!("Request recvd : {:?}", req);

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
        let mut query : String = "SELECT ".to_string();
        query += "a.id as user_id, a.name as user_name, b.password as user_password";
        query += " FROM backoffice_user a, backoffice_user_password b";
        query += " WHERE b.user_id = a.id";
        query += " AND a.name=$1";
        //query += " group by b.date";
        query += " order by b.date";
        println!("query [{}]", query);

        match req.get::<Write<dal::DalPostgresPool>>() {
            Ok(arcpool) => {
                match arcpool.lock() {
                    Ok(x) => {
                        let pool = x.deref();
                        if let Ok(conn) = pool.rw_pool.get() {
                            match conn.prepare(&query) {
                                Ok(stmt) => {
                                    if let Ok(rows) = stmt.query(&[&authuser.username]) {
                                        if rows.is_empty() {
                                            println!("empty rows");
                                        } else {

                                            for row in rows.iter() {

                                                #[derive(Debug, Serialize, Deserialize)]
                                                struct BackOfficeUserAuth {
                                                    pub customer_id_uuid: uuid::Uuid,
                                                    pub password_hash: String,
                                                }

                                                let c: BackOfficeUserAuth = BackOfficeUserAuth {
                                                    customer_id_uuid: row.get("user_id"),
                                                    password_hash: row.get("user_password"),
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
