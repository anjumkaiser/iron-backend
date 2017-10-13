use iron::typemap::Key;


#[derive(Debug)]
pub struct ConfigMisc {
    pub jwt_secret: String,
    pub jwt_time_variation: i64
}


impl Key for ConfigMisc {
    type Value = ConfigMisc;
}
