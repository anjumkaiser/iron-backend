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
        let mut c = Config {
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
            println!("Unable to read config file {}, error {:?}.",
                     config_file_name,
                     e);
            return c;
        }

        let parsed;
        match toml::Parser::new(&config_file_buffer).parse() {
            None => {
                println!("Unable to parse config.");
                return c;
            }
            Some(x) => {
                parsed = x;
            }
        }

        if let Some(x) = parsed.get("server_string") {
            if let Some(y) = x.as_str() {
                if !y.is_empty() {
                    c.server_string = y.to_string();
                }
            }
        }

        if let Some(x) = parsed.get("server") {
            if let Some(server) = x.as_table() {
                if let Some(y) = server.get("ip") {
                    if let Some(z) = y.as_str() {
                        if !z.is_empty() {
                            c.server.ip = z.to_string();
                        }
                    }
                }

                if let Some(y) = server.get("port") {
                    if let Some(z) = y.as_str() {
                        if let Ok(w) = z.parse::<u32>() {
                            if w > 0 {
                                c.server.port = w;
                            }
                        }
                    }
                }

                if let Some(y) = server.get("secure") {
                    if let Some(z) = y.as_bool() {
                        c.server.secure = z;
                    }
                }

                if let Some(y) = server.get("certificate_file") {
                    if let Some(z) = y.as_str() {
                        if !z.is_empty() {
                            c.server.certificate_file = z.to_string();
                        }
                    }
                }

                if let Some(y) = server.get("certificate_password") {
                    if let Some(z) = y.as_str() {
                        if !z.is_empty() {
                            c.server.certificate_password = z.to_string();
                        }
                    }
                }
            }
        }

        if let Some(x) = parsed.get("database") {
            if let Some(database) = x.as_table() {
                if let Some(y) = database.get("url") {
                    if let Some(z) = y.as_str() {
                        if !z.is_empty() {
                            c.database.url = z.to_string();
                        }
                    }
                }

                if let Some(y) = database.get("user") {
                    if let Some(z) = y.as_str() {
                        c.database.user = z.to_string();
                    }
                }

                if let Some(y) = database.get("password") {
                    if let Some(z) = y.as_str() {
                        if !z.is_empty() {
                            c.database.password = z.to_string();
                        }
                    }
                }
            }
        }

        c
    }
}
