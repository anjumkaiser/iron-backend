pub struct Config {
	pub server_string: String,
	pub ip: String,
	pub port: u32
}


impl Config {

	fn new () -> Config {
		let c = Config {
			server_string: "AppName".to_string(),
			ip: "127.0.0.1".to_string(),
			port: 3000
		};

		c
	}

	pub fn load () -> Config {

		let c = Config::new();

		c
	}
}
