use std::fmt;

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> RuntimeError {
        //TODO: have a type?
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

impl RuntimeError {
    pub fn new(msg: &str) -> RuntimeError {
        RuntimeError {
            message: msg.to_string(),
        }
    }

    // pub fn from(error: std::io::Error) -> RuntimeError {
    //     RuntimeError {
    //         message: error.to_string(),
    //     }
    // }

    // pub fn from(msg: &str, error: std::io::Error) -> RuntimeError {
    //     RuntimeError {
    //         message: format!("{}: {}", msg.to_string(), error.to_string()),
    //     }
    // }
}
