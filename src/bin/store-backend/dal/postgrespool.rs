use std::process;

use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use r2d2::Pool;
use iron::typemap::Key;

use config;

pub struct DalPostgresPool {
    pub rw_pool: Pool<PostgresConnectionManager>,
    pub ro_pool: Option<Pool<PostgresConnectionManager>>,
}


impl DalPostgresPool {
    pub fn get_postgres_pool(dbcfg: &config::Config) -> DalPostgresPool {

        let d;
        let config = r2d2::Config::default();
        let manager;

        let mut url = "postgres://".to_string();
        if "" != dbcfg.database.user {
            url += &dbcfg.database.user;
            if "" != dbcfg.database.password {
                url += ":";
                url += &dbcfg.database.password;
            }
            url += "@";
        }
        url += &dbcfg.database.url;

        match PostgresConnectionManager::new(url, TlsMode::None) {
            Ok(value) => {
                manager = value;
            }
            Err(_) => {
                println!("Unable to create Postgres connection manager.");
                process::exit(1);
            }
        }

        match r2d2::Pool::new(config, manager) {
            Err(_) => {
                println!("Unable to create Postgres connection pool.");
                process::exit(1);
            }
            Ok(p) => {
                d = DalPostgresPool {
                    rw_pool: p,
                    ro_pool: None,
                };

                d
            }
        }

    }
}


impl Key for DalPostgresPool {
    type Value = DalPostgresPool;
}
