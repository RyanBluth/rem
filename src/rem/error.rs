use std;
use std::io;
use std::fmt;
use std::error::Error;

use backtrace::Backtrace;

pub const REM_00001: &'static str = "REM_00001: A run mode must be specified. One of [server, client] \
                                 expected";
pub const REM_00002: &'static str = "REM_00002: Unexpected argument encountered";
pub const REM_00003: &'static str = "REM_00003: IO operation failed";
pub const REM_00004: &'static str = "REM_00004: Failed to parse integer value from string";
pub const REM_00005: &'static str = "REM_00005: Invalid key";

/// Simple error structure to be used when errors occur during a cache operation
#[derive(Debug)]
pub struct RemError{
    reason: String
}

impl RemError {
    pub fn with_reason(reason: String) -> RemError {
        return RemError {
            reason: reason
        };
    }

    pub fn with_reason_str(reason: &'static str) -> RemError {
        return RemError {
            reason: String::from(reason)
        };
    }

    pub fn with_reason_str_and_details(reason: &'static str, details: String) -> RemError {
        return RemError {
            reason: String::from(format!("{}: {}\n{:?}", reason, details, Backtrace::new())) 
        };
    }

    pub fn log(self) {
        error!("{}", self);
    }

    pub fn log_and_exit(self) {
        self.log();
        std::process::exit(1);
    }
    
}


impl<'a> Error for RemError {
    fn description(&self) -> &str {
        return &self.reason;
    }
}


impl fmt::Display for RemError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}", self.description());
    }
}

impl From<io::Error> for RemError {
    fn from(e: io::Error) -> RemError {
       return RemError::with_reason_str_and_details(REM_00003, String::from(e.description()));
    }
}


impl From<RemError> for io::Error {
    fn from(e: RemError) -> io::Error {
       let err = io::Error::new(io::ErrorKind::Other, e);
       return err;
    }
}


impl From<std::num::ParseIntError> for RemError {
     fn from(e: std::num::ParseIntError) -> RemError {
       return RemError::with_reason_str_and_details(REM_00004, String::from(e.description()));
    }
}