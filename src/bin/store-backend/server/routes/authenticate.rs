use iron::prelude::*;
use iron::status;
use bodyparser;
use persistent::{Read, Write};
use hyper::header::*;
use hyper::mime::*;
use dal;
use iron::url;
use std;
use std::ops::Deref;
//use r2d2_postgres;
//use time::{Timespec, Tm, at_utc};
use chrono::prelude::*;
use serde_json;
use std::str;
use uuid;
use bcrypt;
use jsonwebtoken;
use slog;
use hyper;

use server::loggerenclave::LoggerEnclave;
use server::routes::ResponseData;
use configmisc;


#[derive(Serialize, Deserialize, Debug)]
pub struct PrivateClaims {
    user_id: uuid::Uuid,
    user_name: String,
    role_id: uuid::Uuid,
    role_name: String,
}

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
}


#[derive(Debug, Serialize, Deserialize)]
struct BackOfficeUserAuth {
    pub user_id: uuid::Uuid,
    pub user_name: String,
    pub role_id: uuid::Uuid,
    pub role_name: String,
    pub password_hash: String,
}

#[inline]
fn create_json_web_token(
    xurl: url::Url,
    remote_addr: String,
    resp_headers: hyper::header::ContentType,
    cfgmisc: std::sync::Arc<configmisc::ConfigMisc>,
    c: BackOfficeUserAuth,
) -> IronResult<Response> {

    let mut resp = Response::with(status::InternalServerError);

    let mut jwt_issuer = String::new();
    {
        let _url: url::Url = xurl;
        jwt_issuer += _url.scheme();

        jwt_issuer += "://";
        if let Some(x) = _url.host_str() {
            //info!(logger, "url host_str [{}]", x);
            jwt_issuer += x;
        }
    }

    let current_time = Utc::now();
    let jwt_issue_timestamp: i64 = current_time.timestamp();
    let jwt_exp_timestamp = jwt_issue_timestamp + (3600 * 1);

    /*
    let jissuetime = Utc.timestamp(jwt_issue_timestamp, 0);
    let jexptime = Utc.timestamp(jwt_exp_timestamp, 0);
    info!(logger, "jwt_iss_time {},  jwet_exp_time {}", jissuetime, jexptime);
    */

    //let remote_addr = .clone();

    let mut jwt_aud = Vec::new();
    jwt_aud.push(c.user_id.simple().to_string());
    jwt_aud.push(c.user_id.to_string());
    jwt_aud.push(c.user_name.clone());
    jwt_aud.push(remote_addr);
    //info!(logger, "jwt_aud {:?}", jwt_aud);


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

    if let Ok(jtoken) = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
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

    resp.headers.set(resp_headers);

    Ok(resp)
}



pub fn backoffice_authenticate(req: &mut Request) -> IronResult<Response> {

    let logger: slog::Logger = get_logger!(req);

    info!(logger, "in backoffice_authenticate");

    let mut resp = Response::with((status::NotFound));

    //let ref rhead = req.headers;
    //info!(logger, "rhead {}", rhead);
    //let ref rbody = req.body;
    //info!(logger, "rbody {}", rbody);

    let mut resp_content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));

    if req.headers.has::<ContentType>() {
        if let Some(ctype) = req.headers.get_raw("content-type") {
            if let Ok(strx) = str::from_utf8(&ctype[0]) {
                info!(logger, "content type received is {}", strx);
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
    info!(logger, "rbody {:?}", rbody);

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct AuthUser {
        pub username: String,
        pub password: String,
    };

    let authuser = match req.get::<bodyparser::Struct<AuthUser>>() {
        Ok(Some(x)) => x,
        _ => {
            info!(logger, "Unable to get AuthUser data from request");
            return Ok(resp);
        }
    };


    info!(logger, "authuser = {:?}", authuser);
    //let query = format!("select * from user where userid={}", authuser.username);
    let mut query: String = "SELECT ".to_string();
    query += "u.id as user_id, u.name as user_name, p.password as user_password, r.id as role_id, r.name as role_name";
    query += " FROM backoffice_user u, backoffice_user_password p, backoffice_user_role r";
    query += " WHERE p.user_id = u.id";
    query += " AND r.id = u.role_id";
    query += " AND u.name=$1";
    query += " order by p.date";
    info!(logger, "query [{}]", query);

    let cfgmisc;

    match req.get::<Read<configmisc::ConfigMisc>>() {
        Ok(dcfgmisc) => {
            cfgmisc = dcfgmisc.clone();
        }
        Err(_) => {
            return Ok(resp);
        }
    }

    let arcpool = match req.get::<Write<dal::DalPostgresPool>>() {
        Ok(x) => x,
        Err(e) => {
            info!(logger, "{:?}", e);
            error!(
                logger,
                "Unable to get connection pool, error message [{}]",
                e
            );
            return Ok(resp);
        }
    };

    let locked_pool = match arcpool.lock() {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "Unable to get lock on connection pool, error message [{}]",
                e
            );
            return Ok(resp);
        }
    };


    let pool = locked_pool.deref();
    let conn = match pool.rw_pool.get() {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "Unable to get connection from pool, error message [{}]",
                e
            );
            return Ok(resp);
        }
    };


    let stmt = match conn.prepare(&query) {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "Unable to get prepare statement, error message [{}]",
                e
            );
            return Ok(resp);
        }
    };


    let rows = match stmt.query(&[&authuser.username]) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to execute query, error message [{}]", e);
            return Ok(resp);
        }
    };


    if rows.is_empty() {
        info!(logger, "No data was found matching requested critera");
        return Ok(resp);
    }

    let mut result: IronResult<Response> = Ok(resp);

    for row in rows.iter() {


        let c: BackOfficeUserAuth = BackOfficeUserAuth {
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            password_hash: row.get("user_password"),
            role_id: row.get("role_id"),
            role_name: row.get("role_name"),
        };
        //info!(logger, "c [{:?}]", c);



        let res = match bcrypt::verify(&authuser.password, &c.password_hash) {
            Ok(x) => x,
            Err(_) => {
                return result;
            }
        };
        //info!(logger, "res [{:?}]", res);

        if true != res {
            break;
        }

        result = create_json_web_token(
            req.url.clone().into(),
            req.remote_addr.to_string(),
            resp_headers,
            cfgmisc,
            c,
        );

        break; // we only need first element
    }

    result
}





pub fn backoffice_renew_token(req: &mut Request) -> IronResult<Response> {

    let logger: slog::Logger = get_logger!(req);

    info!(logger, "in backoffice_renew_token");

    let mut resp = Response::with((status::InternalServerError));

    let rbody = req.get::<bodyparser::Json>();
    info!(logger, "rbody {:?}", rbody);

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct AuthToken {
        pub user_id: String,
        pub token: String,
    };

    let authtoken = match req.get::<bodyparser::Struct<AuthToken>>() {
        Ok(Some(x)) => x,
        _ => {
            info!(logger, "Unable to get AuthUser data from request");
            return Ok(resp);
        }
    };

    let cfgmisc = match req.get::<Read<configmisc::ConfigMisc>>() {
        Ok(dcfgmisc) => dcfgmisc.clone(),
        Err(_) => {
            return Ok(resp);
        }
    };


    let mut validation: jsonwebtoken::Validation = jsonwebtoken::Validation::default();
    validation.leeway = cfgmisc.jwt_time_variation.clone();

    let mut claims: Claims = match jsonwebtoken::decode::<Claims>(
        &authtoken.token,
        cfgmisc.jwt_secret.clone().as_bytes(),
        &validation,
    ) {
        Ok(x) => x.claims,
        Err(_) => return Ok(resp),
    };


    if authtoken.user_id != claims.sub {
        info!(logger, "unmatched user id from token");
        resp = Response::with((status::Unauthorized));
        return Ok(resp);
    }


    let current_time = Utc::now();
    let jwt_issue_timestamp: i64 = current_time.timestamp();
    let jwt_exp_timestamp = jwt_issue_timestamp + (3600 * 1);

    claims.iat = jwt_issue_timestamp;
    claims.exp = jwt_exp_timestamp;
    claims.nbf = jwt_issue_timestamp;

    let jwt_token: String;

    if let Ok(jtoken) = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
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

    Ok(resp)
}


pub fn validate_auth_token(token: String, cfgmisc: configmisc::ConfigMisc) -> Option<PrivateClaims> {

    let mut validation: jsonwebtoken::Validation = jsonwebtoken::Validation::default();
    validation.leeway = cfgmisc.jwt_time_variation.clone();

    match jsonwebtoken::decode::<Claims>(&token, cfgmisc.jwt_secret.clone().as_bytes(), &validation) {
        Ok(x) => Some(x.claims.pvt),
        Err(_) => None,
    }

}
