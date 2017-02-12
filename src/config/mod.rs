#[derive(Deserialize, Debug)]
pub struct Server {
    pub ip: String,
    pub port: u32,
    pub secure: bool,
    pub certificate_file: String,
    pub certificate_password: String,
}


#[derive(Deserialize, Debug)]
pub struct Database {
    pub url: String,
    pub user: String,
    pub password: String,
}


#[derive(Deserialize, Debug)]
pub struct Config {
    pub server_string: String,
    pub server: Server,
    pub database: Database,
}


impl Config {
    fn new() -> Config {
        let c = Config {
            server_string: "AppName".to_string(),
            server: Server {
                ip: "127.0.0.1".to_string(),
                port: 3000,
                secure: false,
                certificate_file: "".to_string(),
                certificate_password: "".to_string(),
            },
            database: Database {
                url: "".to_string(),
                user: "".to_string(),
                password: "".to_string(),
            },
        };

        c
    }


    pub fn load() -> Config {
        use std;
        use std::io::prelude::*;
        use std::fs::File;
        use toml;

        let mut c = Config::new();

        let config_file_name = "config.toml";
        let mut config_file_handle;
        match File::open(config_file_name) {
            Err(e) => {
                println!("Unable to open config file {}, error {:?}.",
                         config_file_name,
                         e);
                return c;
            }
            Ok(value) => {
                config_file_handle = value;
            }
        };

        let mut config_file_buffer = String::new();
        if let Err(e) = config_file_handle.read_to_string(&mut config_file_buffer) {
            println!("Unable to read config file {}, error {}.",
                     config_file_name,
                     e);
            return c;
        }

        match toml::from_str(&config_file_buffer) {
            Err(e) => {
                println!("Unable to parse config. Error returned: {}", e);
                std::process::exit(1);
            }
            Ok(value) => {
                c = value;
            }
        }

        c
    }
}
