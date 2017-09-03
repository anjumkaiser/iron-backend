extern crate uuid;
extern crate postgres;
extern crate lettre;
extern crate common;

use uuid::Uuid;
use postgres::{Connection, TlsMode};


fn main() {

    let c = common::config::Config::load();
    //println!("{:?}", c);
    //println!("Config loaded");

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
        
        let query = "SELECT * FROM email_notifications WHERE status='1'";
        if let Ok(rows) = pgc.query(query, &[]) {
            for row in rows.iter() {
                    let to_address : String =  row.get("to_address");

            }
        }
    }
}
