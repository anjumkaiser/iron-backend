use iron::prelude::*;
use iron::status;
use persistent::{Read, Write};
use hyper::header::*;
use hyper::mime::*;
use dal;
use std::ops::Deref;
use chrono::prelude::*;
use serde_json;
use std::str;

//use uuid;
//use bcrypt;

//use jsonwebtoken::{encode, Header};

//use configmisc;

use slog;

use server::loggerenclave::LoggerEnclave;


pub fn get_products(req: &mut Request) -> IronResult<Response> {

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DbData {
        pub id: i32,
        pub name: String,
        pub timestamp: i64,
    }

    let logger: slog::Logger = get_logger!(req);

    info!(logger, "in get_products");
    let mut resp = Response::with((status::NotFound));

    let arcpool = match req.get::<Write<dal::DalPostgresPool>>() {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "Unable to get connection pool, error message [{}]",
                e
            );
            return Ok(resp);
        }
    };
    let lockedpool = match arcpool.lock() {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "Unable to get lock on connection pool, error message [{}]",
                e
            );
            return Ok(resp);
        }
    };


    let pool = lockedpool.deref();
    let conn = match pool.rw_pool.get() {
        Ok(x) => x,
        Err(e) => {
            error!(
                logger,
                "Unable to get connection from pool, erro message [{}]",
                e
            );
            return Ok(resp);
        }
    };

    let stmt = match conn.prepare("SELECT id, name, FROM PRODUCT") {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to prepare statement, error message [{}]", e);
            return Ok(resp);
        }
    };


    let rows = match stmt.query(&[]) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to execute query, error message [{}]", e);
            return Ok(resp);
        }
    };

    for row in rows.iter() {
        let _id: i32 = row.get("id");
        let _name: String = row.get("name");
        /*// time crate
        let _timestamp: Timespec = row.get("timestamp");
        let utc_tm: Tm = at_utc(_timestamp);
        let local_tm: Tm = utc_tm.to_local();
        info!(
            logger,
            "row [{}, {}, utc {}, local {}] ",
            _id,
            _name,
            utc_tm.asctime(),
            local_tm.asctime()
        );
        */

        // chrono crate
        let _datetime_utc: DateTime<Utc> = row.get("timestamp");
        let _datetime_local: DateTime<Local> = row.get("timestamp");
        info!(logger,
            "row [{}, {}, utc {}, local {}] ",
            _id,
            _name,
            _datetime_utc.to_rfc2822(),
            _datetime_local.to_rfc2822(),
        );

        let data: DbData = DbData {
            id: _id,
            name: _name,
            //timestamp: _timestamp.sec,    // time crate
            timestamp: _datetime_utc.timestamp(), // chrono crate
        };

        if let Ok(json_resp) = serde_json::to_string(&data) {
            resp = Response::with((status::Ok, json_resp));
            resp.headers.set(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![]),
            ));
        };

        break; // we only need first element
    }


    Ok(resp)
}