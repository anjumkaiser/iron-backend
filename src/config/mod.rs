#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
	pub server_string: String,
	pub ip: String,
	pub port: u32,
	pub use_https: bool,
	pub certificate_file: String,
	pub certificate_password: String,
}


impl Config {

	fn new () -> Config {
		let mut c = Config {
			server_string: "AppName".to_string(),
			ip: "127.0.0.1".to_string(),
			port: 3000,
			use_https: true,
			certificate_file: "identity.p12".to_string(),
			certificate_password: "".to_string()
		};

		c.certificate_password = "password".to_string();

		c
	}

	pub fn load () -> Config {

		let c = Config::new();

		c
	}
}
