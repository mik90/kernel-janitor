use std::{fmt, io, num, str};
// Reference: https://learning-rust.github.io/docs/e7.custom_error_types.html
#[derive(Debug)]
pub struct JanitorError {
    kind: String,
    message: String,
}

/// Use like `format!` but it will create a generic JanitorError instead
#[macro_export]
macro_rules! JanitorErrorFrom {
    ($($arg:tt)*) => {{
        JanitorError::from(format!($($arg)*))
    }}
}
/// Use like `format!` but it will create a generic JanitorError wrapped in an Err
#[macro_export]
macro_rules! JanitorResultErr {
    ($($arg:tt)*) => {{
        Err(JanitorError::from(format!($($arg)*)))
    }}
}

impl fmt::Display for JanitorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error type {} occurred with message {}",
            &self.kind, &self.message
        )
    }
}

impl From<&str> for JanitorError {
    fn from(msg: &str) -> Self {
        JanitorError {
            kind: "unknown".to_string(),
            message: msg.to_string(),
        }
    }
}

impl From<String> for JanitorError {
    fn from(msg: String) -> Self {
        JanitorError {
            kind: "unknown".to_string(),
            message: msg,
        }
    }
}

impl From<io::Error> for JanitorError {
    fn from(error: io::Error) -> Self {
        JanitorError {
            kind: String::from("std::io::error"),
            message: error.to_string(),
        }
    }
}

impl From<num::ParseIntError> for JanitorError {
    fn from(error: num::ParseIntError) -> Self {
        JanitorError {
            kind: String::from("std::num::ParseIntError"),
            message: error.to_string(),
        }
    }
}
impl From<str::ParseBoolError> for JanitorError {
    fn from(error: str::ParseBoolError) -> Self {
        JanitorError {
            kind: String::from("std::str::ParseBoolError"),
            message: error.to_string(),
        }
    }
}

impl From<str::Utf8Error> for JanitorError {
    fn from(error: str::Utf8Error) -> Self {
        JanitorError {
            kind: String::from("std::str::Utf8Error"),
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    #[test]
    fn compile_test() {
        let io_err = io::Error::new(std::io::ErrorKind::Other, "some_error");
        let _ = JanitorError::from(io_err);
    }

    #[test]
    fn macro_test() {
        let err_str = "error";
        let _ = JanitorErrorFrom!("Hello, formatted {}", err_str);
    }
}
