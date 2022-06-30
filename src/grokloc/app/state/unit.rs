//! unit implements the State trait for unit testing
use crate::grokloc::app::schema;
use crate::grokloc::app::state;
use crate::grokloc::crypt;
use crate::grokloc::env;
use sqlx;

#[allow(dead_code)]
pub async fn new() -> Result<state::App, state::Err> {
    let master_pool_ = match sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
    {
        Ok(p) => p,
        Err(e) => return Err(state::Err::Sqlx(e)),
    };
    // set up the schema, created anew each time unit state is requested
    let _ = match sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
        .execute(&master_pool_)
        .await
    {
        Ok(o) => o,
        Err(e) => return Err(state::Err::Sqlx(e)),
    };
    let replica_pool_ = master_pool_.clone();
    Ok(state::App {
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
