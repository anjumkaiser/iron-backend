use iron::prelude::*;
use iron::status;

use hyper::header::*;
use hyper::mime::*;

pub fn index_handler(_: &mut Request) -> IronResult<Response> {
	let respdata = "Hello";
	Ok(Response::with((status::Ok, respdata)))
}


pub fn index_handler2 (_: &mut Request) -> IronResult<Response> {
	//let respdata = "Byte Data".as_bytes();
	let respdata = r#"{"key","value"}"#.as_bytes();
	let mut resp = Response::with((status::Ok, respdata));
	resp.headers = Headers::new();
	resp.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
	Ok(resp)
}