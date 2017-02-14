use std::process;

use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use r2d2::Pool;

use config;

pub struct DalPostgresPool {
    pub rw_pool: Pool<PostgresConnectionManager>,
    pub ro_pool: Option<Pool<PostgresConnectionManager>>,
}


impl DalPostgresPool {
    pub fn getPostgresPool(dbcfg: &config::Config) -> DalPostgresPool {

        let d;
        let config = r2d2::Config::default();
        let manager;

        match PostgresConnectionManager::new(dbcfg.database.url.clone(), TlsMode::None) {
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
