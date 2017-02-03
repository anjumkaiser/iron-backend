extern crate iron;
extern crate router;

extern crate hyper;
extern crate params;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;


mod config;
mod server;


fn main() {
	let c = config::Config::load();
	server::serve(c);
}
