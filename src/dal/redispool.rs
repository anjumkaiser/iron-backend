use std::process;

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
    pub fn get_redis_pool(dbcfg: &config::Config) -> DalRedisPool {

        let d;
        let config = r2d2::Config::default();
        let manager;

        let url = dbcfg.redis.url.clone();
        match RedisConnectionManager::new(url.as_str()) {
            Ok(value) => {
                manager = value;
            }
            Err(_) => {
                println!("Unable to create Redis connection manager.");
                process::exit(1);
            }
        }

        match r2d2::Pool::new(config, manager) {
            Err(_) => {
                println!("Unable to create Redis connection pool.");
                process::exit(1);
            }
            Ok(p) => {
                d = DalRedisPool {
                    rw_pool: p,
                    ro_pool: None,
                };

                d
            }
        }

    }
}


impl Key for DalRedisPool {
    type Value = DalRedisPool;
}