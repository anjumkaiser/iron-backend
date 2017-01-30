extern crate iron;
extern crate router;

extern crate hyper;


mod config;
mod server;


fn main() {
	let c = config::Config::load();
	server::serve(c);
}
