//! state provides a trait for accessing conns and symbols
use crate::grokloc::app::schema;
use crate::grokloc::crypt;
use crate::grokloc::env;
use sqlx;
use std::fmt;

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

/// Err abstracts over resource error types
#[derive(Debug)]
pub enum Err {
    Sqlx(sqlx::Error),
}

impl fmt::Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Err::Sqlx(err) => write!(f, "sqlx error {:?}", err),
        }
    }
}

#[allow(dead_code)]
pub async fn unit() -> Result<App, Err> {
    let master_pool_ = match sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
    {
        Ok(p) => p,
        Err(e) => return Err(Err::Sqlx(e)),
    };
    // set up the schema, created anew each time unit state is requested
    let _ = match sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
        .execute(&master_pool_)
        .await
    {
        Ok(o) => o,
        Err(e) => return Err(Err::Sqlx(e)),
    };
    let replica_pool_ = master_pool_.clone();
    Ok(App {
        level: env::Level::Unit,
        master_pool: master_pool_,
        replica_pool: replica_pool_,
        kdf_iterations: 4,
        key: crypt::rand_key(),
        repo_base: String::from("/tmp"),
        root_org: String::from(""),
        root_user: String::from(""),
        root_user_api_secret: String::from(""),
    })
}
