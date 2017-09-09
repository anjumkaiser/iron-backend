extern crate iron;
extern crate router;
extern crate persistent;
extern crate bodyparser;

extern crate hyper;
extern crate hyper_native_tls;
extern crate params;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

extern crate toml;
extern crate uuid;
extern crate bcrypt;

extern crate r2d2;
extern crate r2d2_postgres;
extern crate r2d2_redis;
extern crate postgres;
extern crate redis;

#[macro_use]
extern crate diesel;
extern crate r2d2_diesel;
extern crate dotenv;

extern crate jsonwebtoken;
extern crate url;

extern crate time;
extern crate chrono;

#[macro_use]
extern crate slog;
extern crate slog_json;

extern crate common;

use common::{config, configmisc};

use slog::Drain;

mod server;
mod dal;

fn main() {

    let log_file_nane = "log/store-backend.log";
    let log_file_handle = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(log_file_nane) {
        Ok(x) => x,
        Err(_) => {
            std::process::exit(-1);
        }
    };
    let drain = std::sync::Mutex::new(slog_json::Json::default(log_file_handle));
    let root_logger = slog::Logger::root(
        drain.fuse(),
        o!("version" => env!("CARGO_PKG_VERSION"), "child" => "main"),
    );

    let c = config::Config::load(root_logger.new(o!("child" => "ConfigLoader")));

    let pg_dal = dal::DalPostgresPool::get_postgres_pool(root_logger.new(o!("child" => "DalPostgresPool")), &c);
    // let pg_rw_pool = pg_dal.rw_pool;
    // let pg_ro_pool = pg_dal.ro_pool;
    // let dal::DalPostgresPool { rw_pool: pg_rw_pool, ro_pool: pg_ro_pool } =
    //    dal::DalPostgresPool::get_postgres_pool(&c);

    let redis_dal = dal::DalRedisPool::get_redis_pool(root_logger.new(o!("child" => "DalRedisPool")), &c);

    //let diesel_pg_dal = dal::DalDieselPool::get_diesel_pool(root_logger.new(o!("child" => "DalDieselPool")), &c);

    // SERDE JSON
    // {
    // let json_c = serde_json::to_string(&c).unwrap();
    // println!("c : [{}]", json_c);
    //
    // let des_c: config::Config = serde_json::from_str(&json_c).unwrap();
    // println!("des_c [{:?}", des_c);
    // }
    //

    // TODO  : this should come from an encrypted file
    let config_misc = configmisc::ConfigMisc {
        //jwt_secret: "secretsecret1234567890".to_string()
        //jwt_secret: uuid::Uuid::new_v4().simple().to_string()
        jwt_secret: uuid::Uuid::new_v4().to_string(),
    };

    server::serve(
        root_logger.new(o!("child" => "server")),
        c,
        pg_dal,
        config_misc,
    );

}
