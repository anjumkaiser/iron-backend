use slog;
use iron::prelude::*;
use router::*;
use persistent::{Read, Write};
use config;
use configmisc;
use dal;
//use iron_slog::{LoggerMiddleware, DefaultLogFormatter};

#[macro_use]
mod loggerenclave;
mod routes;


fn configure_router() -> Router {
    let mut router = Router::new();
    router.get("/", routes::index_handler, "index");
    router.get("/2", routes::index_handler2, "index2");
    router.get("/3/:name", routes::index_handler3, "parametric");
    router.get("/getdbtime", routes::get_db_time, "getdbtime");
    router.post("/authenticate", routes::authenticate, "authenticate");
    router.post(
        "/backoffice/authenticate",
        routes::authenticate::local::backoffice_authenticate,
        "backoffice::authenticate",
    );

    router.post(
        "/backoffice/renew",
        routes::authenticate::renew_json_web_token,
        "backoffice::renew_token",
    );

    router.get(
        "/auth/google",
        routes::authenticate::google::google_authenticate,
        "google_authentication",
    );

    router.post("/fileupload", routes::upload_file, "file-uploads");

    router.get("/product", routes::products::get_products, "get_products");
    router.post("/product/add", routes::products::add_product, "add_product");
    router.post("/product/edit/:id", routes::products::edit_product, "edit_product");

    router
}



pub fn serve(logger: slog::Logger, c: config::Config, pg_dal: dal::DalPostgresPool, config_misc: configmisc::ConfigMisc) {

    let router = configure_router();

    /* // TODO: temporarily disabling iron-slog logging
    let logger_formatter = DefaultLogFormatter;
    let logger_middleware = LoggerMiddleware::new(
        router,
        logger.new(o!("child" => "routes")),
        logger_formatter,
    );
    */

    let logger_enclave: loggerenclave::LoggerEnclave = loggerenclave::LoggerEnclave { logger: logger.new(o!("child" => "routes")) };

    //let mut middleware = Chain::new(logger_middleware); // TODO: temporarily disabling iron-slog logging
    let mut middleware = Chain::new(router);
    middleware.link_before(Read::<loggerenclave::LoggerEnclave>::one(logger_enclave));
    middleware.link_before(Read::<configmisc::ConfigMisc>::one(config_misc));
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
                        info!(
                            logger,
                            "{} secure server started, listening on: https://{}/",
                            c.server_string,
                            listening.socket
                        )
                    }
                    Err(e) => error!(logger, "Unable to listen, error message [{}]", e),
                }
            }
            Err(e) => error!(logger, "unable to listen {}", e),
        }
    } else {
        match iron.http(&*ipaddr) {
            Ok(listening) => {
                info!(
                    logger,
                    "{} server started, listening on: http://{}/",
                    c.server_string,
                    listening.socket
                )
            }
            Err(e) => error!(logger, "Unable to listen, error message [{}]", e),
        }
    }

}
