use iron::prelude::*;
use iron::status;
use persistent::{Read, Write};
use hyper::header::*;
use hyper::mime::*;
use bodyparser;
use dal;
use std::ops::Deref;
use chrono::prelude::*;
use serde_json;
use std::str;

//use uuid;
//use bcrypt;

//use jsonwebtoken::{encode, Header};

//use configmisc;
use uuid;
use slog;

use server::loggerenclave::LoggerEnclave;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Product {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub manufacturer_id: uuid::Uuid,
    pub supplier_id: uuid::Uuid,
    pub gtin12: String,
    pub gtin13: String,
}


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

    let stmt = match conn.prepare("SELECT id, name FROM PRODUCT") {
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
        let mut iteration_count = 0;
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


        info!(logger, "iteration_count = {}", iteration_count);
        iteration_count += 1;
        //break; // we only need first element
    }


    Ok(resp)
}

pub fn add_product(req: &mut Request) -> IronResult<Response> {
    let mut resp = Response::with((status::NotFound));
    let logger: slog::Logger = get_logger!(req);

    //{}

    let rbody = req.get::<bodyparser::Json>();
    info!(logger, "rbody {:?}", rbody);

    let product = match req.get::<bodyparser::Struct<Product>>() {
        Ok(Some(x)) => x,
        _ => {
            info!(logger, "Unable to get Product data from request");
            return Ok(resp);
        }
    };
    
    info!(logger, "product = {:?}", product);

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


    let mut str_stmt: String = "INSERT INTO PRODUCT (id, name, description, manufacturer_id, supplier_id, gtin12, gtin13)".to_string();
    str_stmt += " VALUES ($1, $2, $3, $4, $5, $6, $7);";


    let stmt = match conn.prepare(&str_stmt) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to prepare statement, error message [{}]", e);
            return Ok(resp);
        }
    };



    let res = match stmt.execute(&[
            &product.id,
            &product.name,
            &product.description,
            &product.manufacturer_id,
            &product.supplier_id,
            &product.gtin12,
            &product.gtin13
        ]) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to add product into database, error message [{}]", e);
            return Ok(resp);
        }
    };

    info!(logger, "Successfully added product to database {}", product.id);

    resp=Response::with((status::Ok));

    Ok(resp)
}



pub fn edit_product(req: &mut Request) -> IronResult<Response> {
    let mut resp = Response::with((status::NotFound));
    let logger: slog::Logger = get_logger!(req);

    //{}

    let rbody = req.get::<bodyparser::Json>();
    info!(logger, "rbody {:?}", rbody);

    let product = match req.get::<bodyparser::Struct<Product>>() {
        Ok(Some(x)) => x,
        _ => {
            info!(logger, "Unable to get Product data from request");
            return Ok(resp);
        }
    };
    
    info!(logger, "product = {:?}", product);

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


    let mut str_stmt: String = "UPDATE PRODUCT SET name=$2, description=$3, manufacturer_id=$4, supplier_id=$5, gtin12=$6, gtin13=$7)".to_string();
    str_stmt += " WHERE id=$1;";


    let stmt = match conn.prepare(&str_stmt) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to prepare statement, error message [{}]", e);
            return Ok(resp);
        }
    };



    let res = match stmt.execute(&[
            &product.id,
            &product.name,
            &product.description,
            &product.manufacturer_id,
            &product.supplier_id,
            &product.gtin12,
            &product.gtin13
        ]) {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Unable to edit product in database, error message [{}]", e);
            return Ok(resp);
        }
    };

    info!(logger, "Successfully edited product in database {}", product.id);

    resp=Response::with((status::Ok));

    Ok(resp)
}
