use iron::prelude::*;
use iron::status;
use router::*;
use bodyparser;
use persistent::{Read, Write};
use hyper::header::*;
use hyper::mime::*;
use dal;
use std::ops::Deref;
use chrono::prelude::*;
use serde_json;
use std::str;
use uuid;
use bcrypt;
use slog;
use std;
use multipart::server::Multipart;
use multipart::server::MultipartData;
//use multipart::server::{Multipart, Entries, SaveResult, SavedFile};

use server::loggerenclave::LoggerEnclave;

pub mod authenticate;
pub mod products;



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

    let logger: slog::Logger = get_logger!(req);

    info!(logger, "Request recvd : {:?}", req);

    info!(logger, "Url: {:?}", req.url.path());
    if let Some(params) = req.extensions.get::<Router>() {
        info!(logger, "Params {:?}", params["name"]);

        if let Some(name_param) = params.find("name") {
            info!(logger, "Found param name : {}", name_param);
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
        pub id: i32,
        pub name: String,
        pub timestamp: i64,
    }

    let logger: slog::Logger = get_logger!(req);

    info!(logger, "in get_db_time");
    let mut resp = Response::with((status::NotFound));

    let arcpool = match req.get::<Write<dal::DalPostgresPool>>() {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "Unable to get connection pool, error message [{}]",
                e
            );
            return Ok(resp);
        }
    };
    let lockedpool = match arcpool.lock() {
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


    let pool = lockedpool.deref();
    let conn = match pool.rw_pool.get() {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "Unable to get connection from pool, erro message [{}]",
                e
            );
            return Ok(resp);
        }
    };

    let stmt = match conn.prepare("SELECT 1 as id, 'someone' as name, now() as timestamp") {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to prepare statement, error message [{}]", e);
            return Ok(resp);
        }
    };


    let rows = match stmt.query(&[]) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to execute query, error message [{}]", e);
            return Ok(resp);
        }
    };

    for row in rows.iter() {
        let _id: i32 = row.get("id");
        let _name: String = row.get("name");
        /*// time crate
        let _timestamp: Timespec = row.get("timestamp");
        let utc_tm: Tm = at_utc(_timestamp);
        let local_tm: Tm = utc_tm.to_local();
        info!(
            logger,
            "row [{}, {}, utc {}, local {}] ",
            _id,
            _name,
            utc_tm.asctime(),
            local_tm.asctime()
        );
        */

        // chrono crate
        let _datetime_utc: DateTime<Utc> = row.get("timestamp");
        let _datetime_local: DateTime<Local> = row.get("timestamp");
        info!(logger,
            "row [{}, {}, utc {}, local {}] ",
            _id,
            _name,
            _datetime_utc.to_rfc2822(),
            _datetime_local.to_rfc2822(),
        );

        let data: DbData = DbData {
            id: _id,
            name: _name,
            //timestamp: _timestamp.sec,    // time crate
            timestamp: _datetime_utc.timestamp(), // chrono crate
        };

        if let Ok(json_resp) = serde_json::to_string(&data) {
            resp = Response::with((status::Ok, json_resp));
            resp.headers.set(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![]),
            ));
        };

        break; // we only need first element
    }


    Ok(resp)
}



pub fn authenticate(req: &mut Request) -> IronResult<Response> {
    let logger: slog::Logger = get_logger!(req);

    info!(logger, "in authenticate");


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
            error!(logger, "Unable to get authuser from request");
            return Ok(resp);
        }
    };

    info!(logger, "authuser = {:?}", authuser);
    //let query = format!("select * from user where userid={}", authuser.username);
    let query = "select * from customer_local_auth where local_id=$1";
    info!(logger, "query [{}]", query);

    let arcpool = match req.get::<Write<dal::DalPostgresPool>>() {
        Ok(x) => x,
        Err(e) => {
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
            error!(logger, "Unable to prepare statement, error message [{}]", e);
            return Ok(resp);
        }
    };

    let rows = match stmt.query(&[&"admin".to_string()]) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to execute query, error message [{}]", e);
            return Ok(resp);
        }
    };


    if rows.is_empty() {
        info!(logger, "No data was found matching requested critera");
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
            info!(logger, "c [{:?}]", c);
            if let Ok(res) = bcrypt::verify(&authuser.password, &c.password_hash) {
                info!(logger, "res [{:?}]", res);
                if res == true {
                    resp = Response::with((status::Ok));
                }
            }

            break; // we only need first element
        }
    }

    //let ref rbody = req.body;
    //info!(logger, "line #{}", rbody);

    resp.headers.set(resp_content_type);

    Ok(resp)
}





pub fn upload_file(req: &mut Request) -> IronResult<Response> {
    let mut resp = Response::with((status::BadRequest));

    let logger: slog::Logger = get_logger!(req);



    /*
    // params, as it is iron related

    let params = match req.get_ref::<params::Params>() {
        Err(_) => {
            return Ok(resp);
        }
        Ok(x) => x,
    };

    info!(logger, "parsed multipart/params data: {:?}", params);

    match params.find(&["f01", "name"]) {
        Some(&params::Value::String(ref name)) => {
            info!(logger, "data from f01 : {:?}", name);

            //let y = x as File;

            //info!(logger, "f01 {:?}", );
            //info!(logger, "f01 {}", x.size);
        }
        _ => {
            return Ok(resp);
        }

    };
    */


    /*
    let form_data = match formdata::read_formdata(&mut req.body, &req.headers) {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "unable to get files posted in request, error message {}",
                e
            );
            return Ok(resp);
        }
    };

    info!(logger, "read_data {:?}", form_data);

    for file in form_data.files {
        info!(logger, " file ::> {:?}", file);
    }
    */




    /*
    let mut form_data = match Multipart::from_request(req) {
        Ok(x) => x,
        Err(_) => {

            info!(logger, "unable to get multipart");
            return Ok(resp);
        }
    };

    let fd = form_data.save().size_limit(1048576).with_dir(
        std::path::PathBuf::from("/saved/"),
    );

    resp = match fd {
        SaveResult::Full(entries) => {

            info!(logger, "got full entries");

            for (name, file) in entries.files {
                info!(logger, "entry: name {:?} files {:?}", name, file);
                let pth: std::path::PathBuf = file[0].path.clone();
                let mut dpath: std::path::PathBuf = std::path::PathBuf::new();


                if name == "f01" {
                    dpath.push("/saved/");
                    dpath.push(name + ".jpg");

                } else {
                    continue;
                }
                let _ = std::fs::rename(&pth, &dpath);

                info!(
                    logger,
                    "file at path {} copied to {}",
                    pth.to_string_lossy(),
                    dpath.to_string_lossy()
                );
            }

            Response::with((status::Ok))
        }
        SaveResult::Partial(_, reason) => {

            error!(logger, "error fetching files, message {:?}", reason);
            Response::with((status::BadRequest))

        }
        SaveResult::Error(_) => Response::with(status::BadRequest),
    };

    */


    /*
     */
    let mut form_data = match Multipart::from_request(req) {
        Ok(x) => x,
        Err(_) => {

            info!(logger, "unable to get multipart");
            return Ok(resp);
        }
    };

    let multipart_field = match form_data.read_entry() {
        Ok(Some(x)) => x,
        _ => {
            return Ok(resp);
        }
    };


    let fname = multipart_field.name;
    if let MultipartData::File(mut mfile) = multipart_field.data {

        let mut pdest = std::path::PathBuf::new();
        if fname == "f01" {
            pdest.push("/saved/");
            pdest.push(fname + ".jpg");
            mfile.save().with_path(&pdest);

            info!(logger, "file saved to location {}", pdest.to_string_lossy());

            resp.status = Some(status::Ok);
        } else {
            error!(logger, "unexpected file {}, discarding", fname);

        }
    }


    //info!(logger, "read_data {}", form_data);

    //resp.status = Some(status::Ok); // = Response::with((status::Ok));


    Ok(resp)
}
