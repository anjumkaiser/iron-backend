use std::process;

use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel;
use r2d2_diesel::ConnectionManager;
use r2d2::Pool;

use config;

pub struct DalDieselPool {
    pub rw_pool: Pool<r2d2_diesel::ConnectionManager<PgConnection>>,
    //pub ro_pool: Option<Pool<PgConnection>>,
}


impl DalDieselPool {
    pub fn getDieselPool(dbcfg: &config::Config) -> DalDieselPool {

        let d;
        let config = r2d2::Config::default();
        let manager = r2d2_diesel::ConnectionManager::<PgConnection>::new(dbcfg.database.url.clone());

        match r2d2::Pool::new(config, manager) {
            Err(_) => {
                println!("Unable to create Postgres connection pool.");
                process::exit(1);
            }
            Ok(p) => {
                d = DalDieselPool {
                    rw_pool: p,
                    //ro_pool: None,
                };

                d
            }
        }

    }
}
