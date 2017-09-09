use slog;
use std;

use r2d2;
use r2d2_redis::RedisConnectionManager;
use r2d2::Pool;
use iron::typemap::Key;

use config;

pub struct DalRedisPool {
    pub rw_pool: Pool<RedisConnectionManager>,
    pub ro_pool: Option<Pool<RedisConnectionManager>>,
}


impl DalRedisPool {
    pub fn get_redis_pool(logger: slog::Logger, dbcfg: &config::Config) -> DalRedisPool {

        info!(logger, "Creating Postgres connection pool");

        let config = r2d2::Config::default();

        let url = dbcfg.redis.url.clone();
        let manager = match RedisConnectionManager::new(url.as_str()) {
            Ok(x) => x,
            Err(e) => {
                error!(
                    logger,
                    "Unable to create Redis connection manager. error message {}",
                    e
                );
                std::process::exit(1);
            }
        };

        match r2d2::Pool::new(config, manager) {
            Err(e) => {
                error!(
                    logger,
                    "Unable to create Redis connection pool. error message {}",
                    e
                );
                std::process::exit(1);
            }
            Ok(p) => {
                DalRedisPool {
                    rw_pool: p,
                    ro_pool: None,
                }
            }
        }
    }
}


impl Key for DalRedisPool {
    type Value = DalRedisPool;
}
