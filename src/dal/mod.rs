use std::process;

use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use r2d2::Pool;

use config;

pub struct DalPool {
    pub rw_pool: Pool<PostgresConnectionManager>,
    pub ro_pool: Option<Pool<PostgresConnectionManager>>,
}


impl DalPool {
    pub fn get(dbcfg: &config::Config) -> DalPool {

        let d;
        let config = r2d2::Config::default();
        let manager;

        match PostgresConnectionManager::new(dbcfg.database.url.clone(), TlsMode::None) {
            Ok(value) => {
                manager = value;
            }
            Err(_) => {
                process::exit(1);
            }
        }

        match r2d2::Pool::new(config, manager) {
            Err(_) => {
                process::exit(1);
            }
            Ok(p) => {
                d = DalPool {
                    rw_pool: p,
                    ro_pool: None,
                };

                d
            }
        }

    }
}
