use std;
use iron::prelude::*;
use router::*;
use persistent::Write;
use config;
use dal;

mod routes;


fn configure_router() -> Router {
    let mut router = Router::new();
    router.get("/", routes::index_handler, "index");
    router.get("/2", routes::index_handler2, "index2");
    router.get("/3/:name", routes::index_handler3, "parametric");
    router.get("/getdbtime", routes::get_db_time, "getdbtime");
    router.post("/authenticate", routes::authenticate, "authenticate");
    router.post("/backoffice/authenticate", routes::backoffice_authenticate, "backoffice::authenticate");

    router
}



pub fn serve(c: config::Config, pg_dal: dal::DalPostgresPool) {

    let router = configure_router();

    let mut middleware = Chain::new(router);
    middleware.link_before(Write::<dal::DalPostgresPool>::one(pg_dal));

    let iron = Iron::new(middleware);

    let mut ipaddr: String = "".to_string();
    ipaddr += &c.server.ip;
    ipaddr += ":";
    ipaddr += &c.server.port.to_string();

    if c.server.secure {
        use hyper_native_tls;
        match hyper_native_tls::NativeTlsServer::new(c.server.certificate_file, &c.server.certificate_password) {
            Ok(tls) => {
                match iron.https(&*ipaddr, tls) {
                    Ok(listening) => {
                        println!(
                            "{} secure server started, listening on: https://{}/",
                            c.server_string,
                            listening.socket
                        )
                    }
                    Err(e) => println!("Unable to listen, error returned {:?}", e),
                }
            }
            Err(e) => println!("unable to listen {:?}", e),
        }
    } else {
        match iron.http(&*ipaddr) {
            Ok(listening) => {
                println!(
                    "{} server started, listening on: http://{}/",
                    c.server_string,
                    listening.socket
                )
            }
            Err(e) => println!("Unable to listen, error returned {:?}", e),
        }
    }

}
