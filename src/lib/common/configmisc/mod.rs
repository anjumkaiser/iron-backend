use iron::typemap::Key;


#[derive(Debug)]
pub struct ConfigMisc {
    pub jwt_secret: String,
}


impl Key for ConfigMisc {
    type Value = ConfigMisc;
}
