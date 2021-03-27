use std::fmt;

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn new(msg: &str) -> RuntimeError {
        RuntimeError {
            message: msg.to_string(),
        }
    }
}

impl From<hyper::Error> for RuntimeError {
    fn from(error: hyper::Error) -> RuntimeError {
        RuntimeError::new(&error.to_string())
    }
}

impl From<warp::http::Error> for RuntimeError {
    fn from(error: warp::http::Error) -> RuntimeError {
        RuntimeError::new(&error.to_string())
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> RuntimeError {
        RuntimeError::new(&error.to_string())
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {
    fn description(&self) -> &str {
        &self.message
    }
}
