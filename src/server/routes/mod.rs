use iron::prelude::*;
use iron::status;
use router::*;
use persistent::Write;
use hyper::header::*;
use hyper::mime::*;
use dal;
use std::ops::Deref;
use r2d2_postgres;
use time::{Timespec, Tm, at_utc};

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
    println!("in get_db_time");
    let mut resp = Response::with((status::NotFound));

    match req.get::<Write<dal::DalPostgresPool>>() {
        Ok(arcpool) => {
            match arcpool.lock() {
                Ok(x) => {
                    let pool = x.deref();
                    if let Ok(conn) = pool.rw_pool.get() {
                        if let Ok(stmt) = conn.prepare(
                            "SELECT 1 as id, 'someone' as name, now() as timestamp",
                        )
                        {
                            if let Ok(rows) = stmt.query(&[]) {
                                for row in rows.iter() {
                                    let _id: i32 = row.get("id");
                                    let _name: String = row.get("name");
                                    let _timestamp: Timespec = row.get("timestamp");
                                    let utc_tm: Tm = at_utc(_timestamp);
                                    let local_tm: Tm = utc_tm.to_local();
                                    println!("row [{}, {}, utc {}, local {}] ", _id, _name, utc_tm.asctime(), local_tm.asctime());

                                    resp = Response::with(status::Ok);
                                    resp.headers = Headers::new();
                                    resp.headers.set(ContentType(
                                        Mime(TopLevel::Application, SubLevel::Json, vec![]),
                                    ));
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
