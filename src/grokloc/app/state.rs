//! state provides a trait for accessing conns and symbols
#[allow(unused_imports)]
use std::fmt;
use sqlx;
use crate::grokloc::env;

/// StateError abstracts over resource error types
#[derive(Debug)]
#[allow(dead_code)]
pub enum StateError {
    Sqlx(sqlx::Error),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StateError::Sqlx(err) =>
                write!(f, "sqlx error {:?}", err),
        }
    }
}

#[allow(dead_code)]
/// App is the central state access mechanism
pub struct App {
    pub level: env::Level,
    pub master_pool: sqlx::SqlitePool,
    pub replica_pool: sqlx::SqlitePool,
    pub kdf_iterations: u32,
    pub key: String,
    pub repo_base: String,
    pub root_org: String,
    pub root_user: String,
    pub root_user_api_secret: String,
}

// pub mod unit;
