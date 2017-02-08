use iron::prelude::*;
use router::*;
use config;

mod routes;



pub fn serve(c: config::Config) {

	let router = configure_router();
	let iron = Iron::new(router);

	let mut ipaddr: String = "".to_string();
	ipaddr += &c.ip;
	ipaddr += ":";
	ipaddr += &c.port.to_string();

	
	println!("{} server started, listening on {}", c.server_string, ipaddr);

	if c.use_https {
		use hyper_native_tls;
		if let Ok(ssl) = hyper_native_tls::NativeTlsServer::new(c.certificate_file, &c.certificate_password) {
			match iron.https(&*ipaddr, ssl) {
				Ok(listening) => {
					println!("Secure server listening on: {}", listening.socket)
				},
				Err(e) => println!("Unable to listen, error returned {:?}", e)
			}			
		}
	} else {
		match iron.http(&*ipaddr) {
			Ok(listening) => {
				println!("Secure server listening on: {}", listening.socket)
			},
			Err(e) => println!("Unable to listen, error returned {:?}", e)
		}
	}

}


fn configure_router() -> Router {
	let mut router = Router::new();
	router.get("/", routes::index_handler, "index");
	router.get("/2", routes::index_handler2, "index2");
	router.get("/3/:name", routes::index_handler3, "parametric");

	router
}
