#[macro_use]
extern crate slog;
extern crate slog_json;

extern crate uuid;
extern crate postgres;
extern crate native_tls;
extern crate lettre;
extern crate lettre_email;
extern crate mime;
extern crate hostname;
extern crate common;

use uuid::Uuid;
use postgres::{Connection, TlsMode};

use lettre::EmailTransport;
use lettre::smtp::{SmtpTransportBuilder, SmtpTransport};
use lettre::smtp::authentication::Credentials;
use lettre::smtp::authentication::Mechanism;
use lettre::smtp::ConnectionReuseParameters;
use lettre::smtp::SUBMISSION_PORT;
use lettre_email::{EmailBuilder, Email};
use lettre::smtp::extension::ClientId;

use slog::Drain;

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

    let log_file_nane = "log/email-notifier-daemon.log";
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

    info!(logger, "Application started");

    let my_host_name: String;
    if let Some(x) = hostname::get_hostname() {
        my_host_name = x;
        info!(logger, "My hostname: {}", my_host_name);

    } else {
        error!(logger, "Unable to resolv own hostname, quitting");
        std::process::exit(-1);
    }

    let domain: String = "".to_string();
    let native_tls_builder = match native_tls::TlsConnector::builder() {
        Ok(x) => x,
        Err(_) => {
            error!(logger, "Unable to get TLS builder, quitting");
            std::process::exit(-1);
        }
    };
    let native_tls = match native_tls_builder.build() {
        Ok(x) => x,
        Err(_) => {
            error!(logger, "Unable to build TLS connection, quitting");
            std::process::exit(-1);
        }
    };
    let client_tls_params = lettre::smtp::client::net::ClientTlsParameters::new(domain, native_tls);
    let client_security = lettre::smtp::ClientSecurity::Required(client_tls_params);

    let mut mailer_builder;
    if let Ok(x) = SmtpTransportBuilder::new((&c.email_notifier.mailer as &str, SUBMISSION_PORT), client_security) {
        mailer_builder = x;
    } else {
        error!(logger, "Unable to resolv SMTP transport, quitting");
        std::process::exit(-1); // exit program - mailer not foung
    }

    mailer_builder = mailer_builder
        .hello_name(ClientId::Domain(my_host_name))
        .credentials(Credentials::new(c.email_notifier.username, c.email_notifier.password))
        .smtp_utf8(true)
        .authentication_mechanism(Mechanism::CramMd5)
        .connection_reuse(ConnectionReuseParameters::ReuseUnlimited);

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
        error!(logger, "Unable to connect to database, quitting");
        std::process::exit(-1);
    }

    let query = "SELECT * FROM email_notification WHERE status='1'";
    let rows;
    if let Ok(x) = pgc.query(query, &[]) {
        rows = x;
    } else {
        //pgc.close();
        error!(logger, "Unable unable to execute query, quitting");
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
            .subject(mail_data.subject)
            .text(mail_data.mail_body_text)
            .html(mail_data.mail_body_html)
            .build()
        {
            email = x;
        } else {
            continue;
        }

        let send_result = mailer.send(&email);
        if send_result.is_err() {

            error!(logger, "errror sending mail to {}", mail_data.mail_id);
            continue;
        }

        // update database
        let update_statement = "UPDATE email_notifications set status='100' where id='$1'";

        match pgc.execute(update_statement, &[&mail_data.mail_id]) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    logger,
                    "Unable to update status, email id {} will be resent, error message returned {}",
                    mail_data.mail_id,
                    e
                );
            }
        }
    }
}
