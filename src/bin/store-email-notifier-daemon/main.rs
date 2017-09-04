extern crate uuid;
extern crate postgres;
extern crate lettre;
extern crate mime;
extern crate hostname;
extern crate common;

use uuid::Uuid;
use postgres::{Connection, TlsMode};

use lettre::transport::EmailTransport;
use lettre::transport::smtp::{SmtpTransportBuilder, SmtpTransport, SecurityLevel};
use lettre::transport::smtp::authentication::Mechanism;
use lettre::transport::smtp::SUBMISSION_PORT;
use lettre::email::{EmailBuilder, Email};


struct EmailData {
    pub mail_id: Uuid,
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


    let my_host_name: String;
    if let Some(x) = hostname::get_hostname() {
        my_host_name = x;
        println!("My hostname: {}", my_host_name);
    } else {
        std::process::exit(-1);
    }



    let mut mailer_builder;
    if let Ok(x) = SmtpTransportBuilder::new((&c.email_notifier.mailer as &str, SUBMISSION_PORT)) {
        mailer_builder = x;
    } else {
        std::process::exit(-1); // exit program - mailer not foung
    }

    mailer_builder = mailer_builder
        .hello_name(&my_host_name)
        .credentials(&c.email_notifier.username, &c.email_notifier.password)
        .security_level(SecurityLevel::AlwaysEncrypt)
        .smtp_utf8(true)
        .authentication_mechanism(Mechanism::CramMd5)
        .connection_reuse(true);

    let mut mailer: SmtpTransport = mailer_builder.build();


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

    let pgc;
    if let Ok(x) = Connection::connect(connstr, security) {
        pgc = x;
    } else {
        std::process::exit(-1);
    }

    let query = "SELECT * FROM email_notifications WHERE status='1'";
    let rows;
    if let Ok(x) = pgc.query(query, &[]) {
        rows = x;
    } else {
        //pgc.close();
        std::process::exit(-1);
    }

    for row in rows.iter() {

        let mail_data: EmailData = EmailData {
            mail_id: row.get("mail_id"),
            to_address: row.get("to_address"),
            to_address_name: row.get("to_address_name"),
            from_address: row.get("from_address"),
            from_address_name: row.get("from_address_name"),
            subject: row.get("subject"),
            mail_body_html: row.get("mail_body_html"),
            mail_body_text: row.get("mail_body_text"),
            attachments: Vec::new(),
        };


        let email: Email;
        if let Ok(x) = EmailBuilder::new()
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
            email = x;
        } else {
            continue;
        }

        //

        let send_result = mailer.send(email.clone());
        if send_result.is_err() {

            println!("errror sending mail to {}", mail_data.mail_id);
            continue;
        }

        // update database
        let update_statement = "UPDATE email_notifications set status='100' where id='$1'";

        match pgc.execute(update_statement, &[&mail_data.mail_id]) {
            Ok(_) => {}
            Err(e) => {
                println!("error returned {:?}", e);
                let log_message = format!(
                    "Unable to update status, email id will be resent {}",
                    mail_data.mail_id
                );
                println!("{}", log_message);
            }
        }
    }
}
