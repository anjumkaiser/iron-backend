use iron::prelude::*;
use iron::status;
use router::*;
use hyper::header::*;
use hyper::mime::*;
use dal;

pub fn index_handler(_: &mut Request) -> IronResult<Response> {
    let respdata = "Hello";
    Ok(Response::with((status::Ok, respdata)))
}


pub fn index_handler2(_: &mut Request) -> IronResult<Response> {
    let respdata = r#"{"key","value"}"#.as_bytes();
    let mut resp = Response::with((status::Ok, respdata));
    resp.headers = Headers::new();
    resp.headers.set(ContentType(Mime(TopLevel::Application,
                                      SubLevel::Json,
                                      vec![(Attr::Charset, Value::Utf8)])));
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
            resp.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])));
        }
    }

    Ok(resp)
}

pub fn get_db_time(req: &mut Request) -> IronResult<Response> {
    println!("in get_db_time");
    let mut resp = Response::with((status::NotFound));
    match req.extensions.get::<dal::DalPostgresPool>() {
    //match req.get::<dal::DalPostgresPool>() {
        Some(arcpool) => {
            let rwp = arcpool.rw_pool.clone();
            match rwp.get() {
                Ok(conn) => {
                    let qrows = conn.query("SELECT now() as dttm;", &vec![]);
                    println!("qrows [{:?}]", qrows);
                },
                Err(e) => {
                    println!("Unable to get conn, error {:#?}", e);
                }
            }
        },
        None => {
            println!("unable to get arcpool");  
        }
    }

    Ok(resp)
}