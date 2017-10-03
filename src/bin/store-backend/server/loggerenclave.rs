
use slog;
use iron::typemap::Key;

pub struct LoggerEnclave {
    pub logger: slog::Logger,
}

impl Key for LoggerEnclave {
    type Value = LoggerEnclave;
}


macro_rules! get_logger {
    ($req: expr) => (
        //let x: slog::Logger;
        if let Ok(logger_enclave) = $req.get::<Read<LoggerEnclave>>() {
            logger_enclave.clone().logger.new(o!("path" => $req.url.to_string()))
        } else {
            return Ok(Response::with(("", status::BadRequest)));
        }
    )
}
