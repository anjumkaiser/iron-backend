use std::process;

use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use r2d2::Pool;
use iron::typemap::Key;

use config;

pub struct DalDieselPool {
    pub rw_pool: Pool<ConnectionManager<PgConnection>>,
    pub ro_pool: Option<Pool<ConnectionManager<PgConnection>>>,
}


impl DalDieselPool {
    pub fn get_diesel_pool(dbcfg: &config::Config) -> DalDieselPool {

        let d;
        let config = r2d2::Config::default();
        let manager = ConnectionManager::<PgConnection>::new(dbcfg.database.url.clone());

        match r2d2::Pool::new(config, manager) {
            Err(_) => {
                println!("Unable to create Postgres connection pool.");
                process::exit(1);
            }
            Ok(p) => {
                d = DalDieselPool {
                    rw_pool: p,
                    ro_pool: None,
                };

                d
            }
        }

    }
}


impl Key for DalDieselPool {
    type Value = DalDieselPool;
}
