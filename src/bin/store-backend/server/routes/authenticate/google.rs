
use iron::prelude::*;
use iron::status;

/*
use bodyparser;
use persistent::{Read, Write};
use hyper::header::*;
use hyper::mime::*;
use std::ops::Deref;
use std::str;
use bcrypt;
use slog;
*/

//use dal;
//use configmisc;

use self::super::*;


use oauth2;

struct GoogleConfig {
    client_id: String,
    client_secret: String,
    auth_url: String,
    token_url: String,
    redirect_url: String,
}



pub fn google_authenticate(_: &mut Request) -> IronResult<Response> {

    let google_config: GoogleConfig = GoogleConfig {
        client_id: "580129802639-64ek04i3v0ebs09h7s053sa763l797du.apps.googleusercontent.com".to_string(),
        client_secret: "c0LolejtxhNDsyEqrmo3kGgA".to_string(),
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        token_url: "https://www.googleapis.com/oauth2/v3/token".to_string(),
        redirect_url: "http://localhost/login/google/callback".to_string(),
    };


    let GoogleConfig {
        client_id,
        client_secret,
        auth_url,
        token_url,
        redirect_url,
    } = google_config;

    let mut resp: Response; // = Response::with(status::Unauthorized);

    //use oauth2::Config;

    let mut config: oauth2::Config = oauth2::Config::new(client_id, client_secret, auth_url, token_url);


    // add gogle profile option
    config = config
        .add_scope("https://www.googleapis.com/auth/plus.me")
        .set_redirect_url(redirect_url)
        .set_state("1234"); // Set the state parameter (optional)

    // Generate the authorization URL to which we'll redirect the user.
    let authorize_url = config.authorize_url();

    println!(
        "Open this URL in your browser:\n{}\n",
        authorize_url.to_string()
    );
    resp = Response::with(status::TemporaryRedirect);
    resp.headers.set(hyper::header::Location(
        authorize_url.to_string(),
    ));

    //resp.headers.append_raw("url", vec!(authorize_url.to_string());
    //resp = Response::redirect(authorize_url);

    Ok(resp)

}
