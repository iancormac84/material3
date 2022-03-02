use std::fmt;

#[derive(Debug, PartialEq)]
pub struct ArgumentError {
    arg: String,
}

impl ArgumentError {
    pub fn new(arg: String) -> ArgumentError {
        ArgumentError { arg }
    }
}

impl std::fmt::Display for ArgumentError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.arg)
    }
}

impl std::error::Error for ArgumentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}
