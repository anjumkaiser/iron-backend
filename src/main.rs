extern crate iron;
extern crate router;

extern crate hyper;
extern crate params;


mod config;
mod server;


fn main() {
	let c = config::Config::load();
	server::serve(c);
}
