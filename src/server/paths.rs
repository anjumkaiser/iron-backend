use iron::prelude::*;
use iron::status;
use router::*;

use hyper::header::*;
use hyper::mime::*;

pub fn index_handler(_: &mut Request) -> IronResult<Response> {
	let respdata = "Hello";
	Ok(Response::with((status::Ok, respdata)))
}


pub fn index_handler2 (_: &mut Request) -> IronResult<Response> {
	let respdata = r#"{"key","value"}"#.as_bytes();
	let mut resp = Response::with((status::Ok, respdata));
	resp.headers = Headers::new();
	resp.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![(Attr::Charset, Value::Utf8)])));
	Ok(resp)
}

pub fn index_handler3 (req: &mut Request ) -> IronResult<Response> {
	
	let mut resp;

	println!("Request recvd : {:?}", req);

	println!("Url: {:?}", req.url.path());
	if let Some(params) = req.extensions.get::<Router>() {
		//if let Some(query) = params["query"] {
			println!("Params {:?}", params["name"]);
		//}
		
	}
	


	resp = Response::with((status::Ok, "text data"));
	resp.headers = Headers::new();
	resp.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])));


	//resp.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::OctetStream, vec![])));	// allow download

	/*match map.find(&["query"]) {
		Some(&Value::String(ref name)) if name == "Anjum" => {
			let stat = status::Ok;
			let respdata =  "some data".as_bytes();
			resp = Response::with((stat, respdata));
			resp.headers = Headers::new();
			resp.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::OctetStream, vec![])));

		},
		_ => {
			//stat = status::NotFound;
			//respdata = "not found".as_bytes();
			resp = Response::with(status::NotFound)
		}
	}
	*/

	Ok(resp)
}