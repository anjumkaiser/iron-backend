
use slog;
use iron::typemap::Key;

pub struct LoggerEnclave {
    pub logger: slog::Logger,
}

impl Key for LoggerEnclave {
    type Value = LoggerEnclave;
}
