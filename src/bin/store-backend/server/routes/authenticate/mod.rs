use std;
use iron::prelude::*;
use serde_json;
use uuid;
use iron::status;
use iron::url;
use server::routes::ResponseData;
use hyper;
use jsonwebtoken;
use bodyparser;
use chrono::prelude::*;
use slog;
use persistent::Read;
use server::loggerenclave::LoggerEnclave;

use configmisc;



pub mod local;



#[derive(Serialize, Deserialize, Debug)]
pub struct PrivateClaims {
    user_id: uuid::Uuid,
    user_name: String,
    role_id: uuid::Uuid,
    role_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    sub: String, // subject
    iss: String, // issuer
    aud: Vec<String>, // audience
    exp: i64, // expiration time
    nbf: i64, // use not before
    iat: i64, // issued at time
    jti: String, // jwt id - case sensitive and unique among servers
    pvt: PrivateClaims,
}


#[derive(Debug, Serialize, Deserialize)]
struct BackOfficeUserAuth {
    pub user_id: uuid::Uuid,
    pub user_name: String,
    pub role_id: uuid::Uuid,
    pub role_name: String,
    pub password_hash: String,
}

#[inline]
fn create_json_web_token(url: url::Url, remote_addr: String, resp_headers: hyper::header::ContentType, cfgmisc: std::sync::Arc<configmisc::ConfigMisc>, c: BackOfficeUserAuth) -> IronResult<Response> {

    let mut resp = Response::with(status::InternalServerError);

    let mut jwt_issuer = String::new();
    {
        jwt_issuer += url.scheme();

        jwt_issuer += "://";
        if let Some(x) = url.host_str() {
            //info!(logger, "url host_str [{}]", x);
            jwt_issuer += x;
        }
    }

    let current_time = Utc::now();
    let jwt_issue_timestamp: i64 = current_time.timestamp();
    let jwt_exp_timestamp = jwt_issue_timestamp + (3600 * 1);

    /*
    let jissuetime = Utc.timestamp(jwt_issue_timestamp, 0);
    let jexptime = Utc.timestamp(jwt_exp_timestamp, 0);
    info!(logger, "jwt_iss_time {},  jwet_exp_time {}", jissuetime, jexptime);
    */

    //let remote_addr = .clone();

    let mut jwt_aud = Vec::new();
    jwt_aud.push(c.user_id.simple().to_string());
    jwt_aud.push(c.user_id.to_string());
    jwt_aud.push(c.user_name.clone());
    jwt_aud.push(remote_addr);
    //info!(logger, "jwt_aud {:?}", jwt_aud);


    let private_claims = PrivateClaims {
        user_id: c.user_id,
        user_name: c.user_name,
        role_id: c.role_id,
        role_name: c.role_name,
    };

    let my_claims = Claims {
        sub: c.user_id.simple().to_string(), // populating uuid as simple format (hypen-less format) string
        iss: jwt_issuer,
        aud: jwt_aud,
        iat: jwt_issue_timestamp,
        exp: jwt_exp_timestamp,
        jti: uuid::Uuid::new_v4().simple().to_string(),
        nbf: jwt_issue_timestamp,
        pvt: private_claims,
    };

    let jwt_token: String;

    if let Ok(jtoken) = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &my_claims,
        &cfgmisc.jwt_secret.clone().into_bytes(),
    )
    {
        jwt_token = jtoken;
    } else {
        resp.headers.set(resp_headers);
        return Ok(resp);
    }

    let resp_data: ResponseData<String> = ResponseData {
        success: true,
        data: jwt_token,
        message: "".to_owned(),
    };

    if let Ok(respjson) = serde_json::to_string(&resp_data) {
        resp = Response::with((status::Ok, respjson));
    }

    resp.headers.set(resp_headers);

    Ok(resp)
}



pub fn renew_json_web_token(req: &mut Request) -> IronResult<Response> {

    let logger: slog::Logger = get_logger!(req);

    info!(logger, "in renew_json_web_token");

    let mut resp = Response::with((status::InternalServerError));

    let rbody = req.get::<bodyparser::Json>();
    info!(logger, "rbody {:?}", rbody);

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct AuthToken {
        pub user_id: String,
        pub token: String,
    };

    let authtoken = match req.get::<bodyparser::Struct<AuthToken>>() {
        Ok(Some(x)) => x,
        _ => {
            info!(logger, "Unable to get AuthUser data from request");
            return Ok(resp);
        }
    };

    let cfgmisc = match req.get::<Read<configmisc::ConfigMisc>>() {
        Ok(dcfgmisc) => dcfgmisc.clone(),
        Err(_) => {
            return Ok(resp);
        }
    };


    let mut validation: jsonwebtoken::Validation = jsonwebtoken::Validation::default();
    validation.leeway = cfgmisc.jwt_time_variation.clone();

    let mut claims: Claims = match jsonwebtoken::decode::<Claims>(
        &authtoken.token,
        cfgmisc.jwt_secret.clone().as_bytes(),
        &validation,
    ) {
        Ok(x) => x.claims,
        Err(_) => return Ok(resp),
    };


    if authtoken.user_id != claims.sub {
        info!(logger, "unmatched user id from token");
        resp = Response::with((status::Unauthorized));
        return Ok(resp);
    }


    let current_time = Utc::now();
    let jwt_issue_timestamp: i64 = current_time.timestamp();
    let jwt_exp_timestamp = jwt_issue_timestamp + (3600 * 1);

    claims.iat = jwt_issue_timestamp;
    claims.exp = jwt_exp_timestamp;
    claims.nbf = jwt_issue_timestamp;

    let jwt_token: String;

    if let Ok(jtoken) = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &cfgmisc.jwt_secret.clone().into_bytes(),
    )
    {
        jwt_token = jtoken;
    } else {
        return Ok(resp);
    }

    let resp_data: ResponseData<String> = ResponseData {
        success: true,
        data: jwt_token,
        message: "".to_owned(),
    };

    if let Ok(respjson) = serde_json::to_string(&resp_data) {
        resp = Response::with((status::Ok, respjson));
    }

    Ok(resp)
}



pub fn validate_auth_token(token: String, cfgmisc: configmisc::ConfigMisc) -> Option<PrivateClaims> {

    let mut validation: jsonwebtoken::Validation = jsonwebtoken::Validation::default();
    validation.leeway = cfgmisc.jwt_time_variation.clone();

    match jsonwebtoken::decode::<Claims>(&token, cfgmisc.jwt_secret.clone().as_bytes(), &validation) {
        Ok(x) => Some(x.claims.pvt),
        Err(_) => None,
    }

}
