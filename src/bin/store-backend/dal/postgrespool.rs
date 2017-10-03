use slog;
use std;
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
    pub fn get_postgres_pool(logger: slog::Logger, dbcfg: &config::Config) -> DalPostgresPool {

        info!(logger, "Creating Postgres connection pool");

        let config = r2d2::Config::default();

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

        let manager = match PostgresConnectionManager::new(url, TlsMode::None) {
            Ok(x) => x,
            Err(e) => {
                error!(
                    logger,
                    "Unable to create Postgres connection manager, error message [{}]",
                    e
                );
                std::process::exit(1);
            }
        };

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
                info!(logger, "Successfully created connection pool");
                DalPostgresPool {
                    rw_pool: p,
                    ro_pool: None,
                }
            }
        }
    }
}


impl Key for DalPostgresPool {
    type Value = DalPostgresPool;
}
