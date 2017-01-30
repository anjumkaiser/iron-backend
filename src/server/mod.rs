use config::*;

use iron::prelude::*;
use router::*;

mod paths;

pub fn serve(c: Config) {

	let router = configure_router();
	let iron = Iron::new(router);

	let mut ipaddr: String = "".to_string();
	ipaddr += &c.ip;
	ipaddr += ":";
	ipaddr += &c.port.to_string();

	//println!("{} : {}", c.ip, c.port);

	println!("{} server started, listening on {}", c.server_string, ipaddr);
	match iron.http(&*ipaddr) {
		Ok(listening) => println!("Result: {:?}", listening),
		Err(x) => println!("Unable to listen, error returned {:?}", x)
	}
}


fn configure_router() -> Router {
	let mut router = Router::new();
	router.get("/", paths::index_handler, "index");
	router.get("/2", paths::index_handler2, "index2");
	router.get("/3/:name", paths::index_handler3, "parametric");

	router
}
