use iron::prelude::*;
use iron::status;


pub fn index_handler(_: &mut Request) -> IronResult<Response> {
	let respdata = "Hello";
	Ok(Response::with((status::Ok, respdata)))
}


pub fn index_handler2 (_: &mut Request) -> IronResult<Response> {
	let respdata = "Byte Data".as_bytes();
	Ok(Response::with((status::Ok, respdata)))
}