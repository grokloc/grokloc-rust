//! state provides a trait for accessing conns and symbols
use crate::grokloc::app::admin::org::Org;
use crate::grokloc::app::admin::user::User;
use crate::grokloc::app::schema;
use crate::grokloc::crypt;
use crate::grokloc::env;
use crate::grokloc::safe;
use anyhow;
use sqlx;

/// App is the central state access mechanism
pub struct App {
    pub level: env::Level,
    pub master_pool: sqlx::SqlitePool,
    pub replica_pool: sqlx::SqlitePool,
    pub kdf_iterations: u32,
    pub key: String,
    pub repo_base: String,
    pub root_org: Org,
    pub root_user: User,
}

#[allow(dead_code)]
pub async fn unit() -> Result<App, anyhow::Error> {
    let master_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await?;
    sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
        .execute(&master_pool)
        .await?;

    let key = crypt::rand_key();
    let (root_org, root_user) = Org::create(
        &master_pool,
        &safe::VarChar::rand(), // org name
        &safe::VarChar::rand(), // org owner display name
        &safe::VarChar::rand(), // org owner email
        &safe::VarChar::new(&crypt::kdf(&crypt::rand_hex(), crypt::MIN_KDF_ROUNDS))?, // org owner password
        &key,
    )
    .await?;

    let replica_pool = master_pool.clone();
    Ok(App {
        level: env::Level::Unit,
        master_pool,
        replica_pool,
        kdf_iterations: crypt::MIN_KDF_ROUNDS,
        key,
        repo_base: String::from("/tmp"),
        root_org,
        root_user,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow;

    #[tokio::test]
    async fn unit_state_test() -> Result<(), anyhow::Error> {
        let _ = unit().await?;
        Ok(())
    }
}
