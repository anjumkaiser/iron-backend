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

	// SERDE JSON
	{
		let json_c = serde_json::to_string(&c).unwrap();
		println!("c : [{}]", json_c);

		let des_c: config::Config = serde_json::from_str(&json_c).unwrap();
		println!("des_c [{:?}", des_c);
	}

	server::serve(c);
}
