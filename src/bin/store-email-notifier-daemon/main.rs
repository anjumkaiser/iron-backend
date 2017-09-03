extern crate uuid;
extern crate postgres;
extern crate lettre;
extern crate mime;
extern crate common;

use uuid::Uuid;
use postgres::{Connection, TlsMode};


struct EmailData {
    pub from_address: String,
    pub from_address_name: String,
    pub to_address: String,
    pub to_address_name: String,
    pub subject: String,
    pub mail_body_text: String,
    pub mail_body_html: String,
    pub attachments: Vec<String>,
}

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

                let mail_data: EmailData = EmailData {
                    to_address: row.get("to_address"),
                    to_address_name: row.get("to_address_name"),
                    from_address: row.get("from_address"),
                    from_address_name: row.get("from_address_name"),
                    subject: row.get("subject"),
                    mail_body_html: row.get("mail_body_html"),
                    mail_body_text: row.get("mail_body_text"),
                    attachments: Vec::new()
                };

                use lettre::email::{EmailBuilder, Email};
                let mut email;
                if let Ok(_email) = EmailBuilder::new()
                    .to((
                        &mail_data.to_address as &str,
                        &mail_data.to_address_name as &str,
                    ))
                    .from((
                        &mail_data.from_address as &str,
                        &mail_data.from_address_name as &str,
                    ))
                    .subject(&mail_data.subject)
                    .text(&mail_data.mail_body_text)
                    .html(&mail_data.mail_body_html)
                    .build()
                {
                    email = _email;
                } else {
                    continue;
                }

            }
        }
    }
}
