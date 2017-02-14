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
extern crate postgres;

mod config;
mod server;
mod dal;

fn main() {

    let c = config::Config::load();
    let dal = dal::DalPool::get(&c);
    let ro_pool = dal.ro_pool;
    let rw_pool = dal.rw_pool;

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
