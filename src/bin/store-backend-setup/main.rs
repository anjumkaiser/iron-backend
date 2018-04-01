extern crate uuid;
extern crate bcrypt;
extern crate postgres;
extern crate rpassword;
extern crate rprompt;

#[macro_use]
extern crate slog;
extern crate slog_json;

extern crate common;

use uuid::Uuid;
use postgres::{Connection, TlsMode};

use slog::Drain;

fn main() {

    let log_file_nane = "log/setup.log";
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
    let c = common::config::Config::load(root_logger.new(o!("child" => "Config")));
    let logger = root_logger;

    info!(logger, "Config loaded");

    let account_status_active_id = 1;
    let account_status_active_name = "Active";

    let administrator_group_id = Uuid::new_v4();
    let administrator_group_name = "Administrator";

    let administrator_user_id = Uuid::new_v4();

    let administrator_user_name = match rprompt::prompt_reply_stdout("Please provide user name for administrator: ") {
        Ok(x) => x,
        Err(e) => {
            error!(logger, "Error reading username, error message {}", e);
            std::process::exit(1);
        }
    };

    let password = match rpassword::prompt_password_stdout(&format!(
        "Please enter password for user '{}': ",
        administrator_user_name
    )) {

        Ok(x) => x,
        Err(e) => {
            error!(logger, "Error reading password, error message {}", e);
            std::process::exit(1);
        }
    };

    let administrator_user_password_hash = match bcrypt::hash(&password, c.password_hash_cost) {
        Ok(pw_hash) => {
            info!(logger, "Successfully encrypted password");
            pw_hash
        }
        Err(e) => {
            error!(
                logger,
                "Unable to encrypt password, quiting, error message {}",
                e
            );
            std::process::exit(1);
        }
    };

    let mut connstr = "postgres://".to_string();
    if "" != c.database.user {
        connstr += &c.database.user;
        if "" != c.database.password {
            connstr += ":";
            connstr += &c.database.password;
        }
        connstr += "@";
    }
    connstr += &c.database.url;

    let security = TlsMode::None;

    if let Ok(pgc) = Connection::connect(connstr, security) {

        let mut res;

        if let Ok(txn) = pgc.transaction() {

            let mut should_commit: bool = true;


            res = txn.execute(
                "INSERT INTO backoffice_user_status (id, name) VALUES ($1, $2)",
                &[&account_status_active_id, &account_status_active_name],
            );

            match res {
                Ok(_) => {
                    info!(logger, "Added backoffice_user_status");
                }
                Err(e) => {
                    error!(logger, "Unable to add backoffice_user_status, {}", e);
                    should_commit = false;
                }
            }

            res = txn.execute(
                "INSERT INTO backoffice_user_role (id, name) VALUES ($1, $2)",
                &[&administrator_group_id, &administrator_group_name],
            );

            match res {
                Ok(_) => {}
                Err(e) => {
                    error!(logger, "Unable to add backoffice_user_role, {}", e);
                    should_commit = false;
                }
            }

            res = txn.execute(
                "INSERT INTO backoffice_user (id, name, role_id, status) VALUES ($1, $2, $3, $4)",
                &[
                    &administrator_user_id,
                    &administrator_user_name,
                    &administrator_group_id,
                    &1,
                ],
            );

            match res {
                Ok(_) => {
                    info!(
                        logger,
                        "Successfully added backoffice user id {} name {} groupid {}",
                        &administrator_user_id,
                        &administrator_user_name,
                        &administrator_group_id
                    );
                }
                Err(e) => {
                    error!(logger, "Unable to add backoffice_user, {}", e);
                    should_commit = false;
                }
            }

            res = txn.execute(
                "INSERT INTO backoffice_user_password (user_id, password, date) VALUES ($1, $2, now())",
                &[&administrator_user_id, &administrator_user_password_hash],
            );

            match res {
                Ok(_) => {
                    info!(logger, "Successfully updated user password");
                }
                Err(e) => {
                    error!(logger, "Unable to add backoffice_user_password, {}", e);
                    should_commit = false;
                }
            }

            res = txn.execute("INSERT INTO country (code, name) VALUES ('000', 'Unknown')", &[]);

            match res {
                Ok(_) => {
                    info!(logger, "Successfully added unknown country");
                }
                Err(e) => {
                    error!(logger, "Unable to add unknown country, {}", e);
                    should_commit = false;
                }
            }


            let city_id = Uuid::new_v4();
            info!(logger, "city id is: {}", city_id);

            res = txn.execute("INSERT INTO city (id, name, country_code) VALUES ($1, 'Unknown', '000')", &[&city_id]);

            match res {
                Ok(_) => {
                    info!(logger, "Successfully added unknown city");
                }
                Err(e) => {
                    error!(logger, "Unable to add unknown city, {}", e);
                    should_commit = false;
                }
            }


            res = txn.execute("INSERT INTO manufacturer (id, name, address, principal_contact) VALUES ($1, 'Unknown', 'Unknown', 'Unknown')", &[&Uuid::new_v4()]);

            match res {
                Ok(_) => {
                    info!(logger, "Successfully added unknown manufacturer");
                }
                Err(e) => {
                    error!(logger, "Unable to add unknown manufacturer, {}", e);
                    should_commit = false;
                }
            }


            res = txn.execute("INSERT INTO supplier (id, name, address, city, principal_contact) VALUES ($1, 'Unknown', 'Unknown', $2, 'Unknown')", &[&Uuid::new_v4(), &city_id]);

            match res {
                Ok(_) => {
                    info!(logger, "Successfully added unknown supplier");
                }
                Err(e) => {
                    error!(logger, "Unable to add unknown supplier, {}", e);
                    should_commit = false;
                }
            }

            if should_commit {
                txn.set_commit();
            } else {
                error!(logger, "All transactions will be rolled back.");
            }

            match txn.finish() {
                Ok(_) => {
                    info!(logger, "Transaction operation completed succesfully.");
                }
                Err(e) => {
                    info!(
                        logger,
                        "Transaction operation completion unsuccessful,  {}",
                        e
                    );
                }
            }
        } else {
            error!(logger, "Unable to begin transaction, quitting");
            std::process::exit(1);
        }
    } else {
        error!(logger, "Unable to connect to database, quitting");
        std::process::exit(1);
    }
}
