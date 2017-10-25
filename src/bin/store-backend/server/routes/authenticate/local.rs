use iron::prelude::*;
use iron::status;
use bodyparser;
use persistent::{Read, Write};
use hyper::header::*;
use hyper::mime::*;
use std::ops::Deref;
use std::str;
use bcrypt;
use slog;

use dal;
use configmisc;

use self::super::*;



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
