extern crate iron;
extern crate router;

extern crate hyper;
extern crate hyper_native_tls;
extern crate params;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate toml;

extern crate r2d2;
extern crate r2d2_postgres;
extern crate r2d2_redis;
extern crate postgres;
extern crate redis;

#[macro_use]
extern crate diesel;
extern crate r2d2_diesel;
extern crate dotenv;

mod config;
mod server;
mod dal;

fn main() {

    let c = config::Config::load();
    let pg_dal = dal::DalPostgresPool::getPostgresPool(&c);
    let pg_ro_pool = pg_dal.ro_pool;
    let pg_rw_pool = pg_dal.rw_pool;

    let redis_dal = dal::DalRedisPool::getRedisPool(&c);

    let diesel_pg_dal = dal::DalDieselPool::getDieselPool(&c);

    // SERDE JSON
    // {
    // let json_c = serde_json::to_string(&c).unwrap();
    // println!("c : [{}]", json_c);
    //
    // let des_c: config::Config = serde_json::from_str(&json_c).unwrap();
    // println!("des_c [{:?}", des_c);
    // }
    //

    server::serve(c);

}
