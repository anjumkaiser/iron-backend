extern crate iron;
extern crate router;


mod config;
mod server;


fn main() {
	let c = config::Config::load();
	server::serve(c);
}
