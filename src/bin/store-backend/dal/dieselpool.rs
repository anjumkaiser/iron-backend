use slog;
use std;

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
    pub fn get_diesel_pool(logger: slog::Logger, dbcfg: &config::Config) -> DalDieselPool {

        info!(logger, "Creating Diesel pool");

        let config = r2d2::Config::default();
        let manager = ConnectionManager::<PgConnection>::new(dbcfg.database.url.clone());

        match r2d2::Pool::new(config, manager) {
            Err(e) => {
                error!(
                    logger,
                    "Unable to create Postgres connection pool. error message [{}]",
                    e
                );
                std::process::exit(1);
            }
            Ok(p) => {
                DalDieselPool {
                    rw_pool: p,
                    ro_pool: None,
                }
            }
        }
    }
}


impl Key for DalDieselPool {
    type Value = DalDieselPool;
}
