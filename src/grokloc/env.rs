//! env contains environment specific functions and symbols
use std::fmt;
use thiserror::Error;

#[allow(dead_code)]
pub const GROKLOC_ENV_KEY: &str = "GROKLOC_ENV";

/// Level describes the run level
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Level {
    Unit,
}

impl Default for Level {
    fn default() -> Self {
        Level::Unit
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Err covers level errors
#[derive(Debug, Error, PartialEq)]
#[allow(dead_code)]
pub enum Err {
    #[error("unknown level")]
    UnknownLevel,
}
