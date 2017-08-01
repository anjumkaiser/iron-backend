extern crate uuid;
extern crate bcrypt;
extern crate postgres;
extern crate rpassword;
extern crate rprompt;

extern crate common;

use uuid::Uuid;
use postgres::{Connection, TlsMode};


fn main() {

    let c = common::config::Config::load();
    //println!("{:?}", c);
    //println!("Config loaded");

    let account_status_active_id = 1;
    let account_status_active_name = "Active";

    let administrator_group_id = Uuid::new_v4();
    let administrator_group_name = "Administrator";

    let administrator_user_id = Uuid::new_v4();
    let administrator_user_name;

    //let mut password: String = String::new();
    let password;

    //let mut administrator_user_password_hash: String = String::new();
    let administrator_user_password_hash;


    match rprompt::prompt_reply_stdout("Please provide user name for administrator: ") {
        Err(_) => {
            ::std::process::exit(1);
        }
        Ok(s) => {
            administrator_user_name = s;
        }
    }


    match rpassword::prompt_password_stdout(&format!(
        "Please neter password for user '{}': ",
        administrator_user_name
    )) {

        Ok(x) => password = x,
        Err(e) => {
            println!("Error reading password {}", e);
            ::std::process::exit(1);
        }

    }
    match bcrypt::hash(&password, c.password_hash_cost) {
        Ok(pw_hash) => {
            administrator_user_password_hash = pw_hash;
        }
        _ => {
            println!("Unable to read password / password error, quiting");
            ::std::process::exit(1);
        }
    }

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
                    //println!("{:?}", e);
                }
                Err(e) => {
                    //println!("{:?}", e);
                    println!("Unable to add backoffice_user_status, {}", e);
                    should_commit = false;
                }
            }

            res = txn.execute(
                "INSERT INTO backoffice_user_role (id, name) VALUES ($1, $2)",
                &[&administrator_group_id, &administrator_group_name],
            );

            match res {
                Ok(_) => {
                    //println!("{:?}", e);
                }
                Err(e) => {
                    //println!("{:?}", e);
                    println!("Unable to add backoffice_user_role, {}", e);
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
                    //println!("{:?}", e);
                }
                Err(e) => {
                    //println!("{:?}", e);
                    println!("Unable to add backoffice_user_password, {}", e);
                    should_commit = false;
                }
            }


            res = txn.execute(
                "INSERT INTO backoffice_user_password (user_id, password, date) VALUES ($1, $2, now())",
                &[&administrator_user_id, &administrator_user_password_hash],
            );

            match res {
                Ok(_) => {
                    //println!("{:?}", e);
                }
                Err(e) => {
                    println!("Unable to add backoffice_user_password, {}", e);
                    //println!("{:?}", e);
                    should_commit = false;
                }
            }


            if should_commit {
                txn.set_commit();
            //println!("txn will commit");
            } else {
                println!("All transactions will be rolled back.");
            }

            match txn.finish() {
                Ok(_) => {
                    println!("Transaction operation completed succesfully.");
                }
                Err(e) => {
                    println!("Transaction operation completion unsuccessful,  {:?}", e);
                }
            }

        }

    }

}
