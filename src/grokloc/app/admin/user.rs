//! user models an orgs row and related db functionality
use crate::grokloc::app::models;
use crate::grokloc::crypt;
use crate::grokloc::db;
use crate::grokloc::safe;
use sqlx;
use std::error::Error;
use uuid::Uuid;

pub const SCHEMA_VERSION: i8 = 0;

pub const INSERT_QUERY: &str = r#"
insert into users
(id,
 api_secret,
 api_secret_digest,
 display_name,
 display_name_digest,
 email,
 email_digest,
 org,
 password,
 status,
 schema_version)
 values
(?,?,?,?,?,?,?,?,?,?,?)
"#;

#[allow(dead_code)]
pub const SELECT_QUERY: &str = r#"
    select
    api_secret,
    api_secret_digest,
    display_name,
    display_name_digest,
    email,
    email_digest,
    org,
    password,
    schema_version,
    status,
    ctime,
    mtime
    from users where id = ?;
"#;

/// User is the data representation of an users row
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct User {
    pub id: Uuid,
    pub api_secret: safe::VarChar,
    pub api_secret_digest: safe::VarChar,
    pub display_name: safe::VarChar,
    pub display_name_digest: safe::VarChar,
    pub email: safe::VarChar,
    pub email_digest: safe::VarChar,
    pub org: Uuid,
    pub password: safe::VarChar,
    pub meta: models::Meta,
}

/// encrypted makes a new User with PII fields encrypted with key as key
/// and salt derived from the email (which is constrained to be unique in the db)
#[allow(dead_code)]
pub fn encrypted(
    display_name: &safe::VarChar,
    email: &safe::VarChar,
    org: &Uuid,
    password: &safe::VarChar, // assumed already derived
    key: &str,
) -> Result<User, Box<dyn Error>> {
    let email_digest = crypt::sha256_hex(&email.to_string());
    let iv = crypt::iv(&email_digest);
    let api_secret_ = Uuid::new_v4();

    Ok(User {
        id: Uuid::new_v4(),
        api_secret: safe::VarChar::new(&crypt::encrypt(key, &iv, &api_secret_.to_string())?)?,
        api_secret_digest: safe::VarChar::new(&crypt::sha256_hex(&api_secret_.to_string()))?,
        display_name: safe::VarChar::new(&crypt::encrypt(key, &iv, &display_name.to_string())?)?,
        display_name_digest: safe::VarChar::new(&crypt::sha256_hex(&display_name.to_string()))?,
        email: safe::VarChar::new(&crypt::encrypt(key, &iv, &email.to_string())?)?,
        email_digest: safe::VarChar::new(&email_digest)?,
        org: *org,
        password: password.clone(),
        meta: models::Meta {
            schema_version: SCHEMA_VERSION,
            ..Default::default()
        },
    })
}

impl User {
    /// insert performs db insert with no integrity check on the org (see "create")
    ///
    /// assumed to be called within an existing transaction that includes
    /// consistency checks, so connection handle is a sqlx::Transaction
    #[allow(dead_code)]
    pub async fn insert(
        &self,
        txn: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<(), db::Err> {
        if let Err(insert_error) = sqlx::query(INSERT_QUERY)
            .bind(self.id.to_string())
            .bind(self.api_secret.to_string())
            .bind(self.api_secret_digest.to_string())
            .bind(self.display_name.to_string())
            .bind(self.display_name_digest.to_string())
            .bind(self.email.to_string())
            .bind(self.email_digest.to_string())
            .bind(self.org.to_string())
            .bind(self.password.to_string())
            .bind(self.meta.status.to_int())
            .bind(self.meta.schema_version)
            .execute(txn)
            .await
        {
            return Err(db::Err::SQLx(insert_error)); // implicit rollback
        }
        Ok(())
    }

    // pub async fn read(pool: sqlx::SqlitePool, id: &str, key: &str) -> Result<Self, db::Err> {
    //     let row = match sqlx::query(SELECT_QUERY).bind(id).fetch_one(pool).await {
    //         Err(error) => return Err(db::Err::SQLx(error)),
    //         Ok(v) => v,
    //     };
    //     let email_digest_ = match row.try_get::<String, _>("email_digest") {
    //         Err(error) => return Err(db::Err::SQLx(error)),
    //         Ok(v) => v,
    //     };
    //     let iv = crypt::iv_truncate(&email_digest);
    //     let encrypted_api_secret = match row.try_get::<String, _>("api_secret") {
    //         Err(error) => return Err(db::Err::SQLx(error)),
    //         Ok(v) => v,
    //     };
    //     let api_secret_ = match crypt::decrypt(key, &iv, &encrypted_api_secret) {
    //         Ok(c) => c,
    //         Err(error) => panic!("decrypt api secret: {:?}", error),
    //     };
    //     let encrypted_display_name = match row.try_get::<String, _>("display_name") {
    //         Err(error) => return Err(db::Err::SQLx(error)),
    //         Ok(v) => v,
    //     };
    //     let display_name_ = match crypt::decrypt(key, &iv, &encrypted_display_name) {
    //         Ok(c) => c,
    //         Err(error) => panic!("decrypt display name: {:?}", error),
    //     };
    //     let encrypted_email = match row.try_get::<String, _>("email") {
    //         Err(error) => return Err(db::Err::SQLx(error)),
    //         Ok(v) => v,
    //     };
    //     let email_ = match crypt::decrypt(key, &iv, &encrypted_email) {
    //         Ok(c) => c,
    //         Err(error) => panic!("decrypt email: {:?}", error),
    //     };
    //     Ok(Self {
    //         id: Uuid::try_parse(row.try_get::<String, _>("id")?)?,
    //         api_secret: Uuid::try_parse(&api_secret_)?,
    //         display_name: safe::VarChar::new(&display_name_)?,
    //         display_name_digest: safe::VarChar::new(
    //             &row.try_get::<String, _>("display_name_digest")?,
    //         )?,
    //         email: safe::VarChar::new(&email_)?,
    //         email_digest: safe::VarChar::new(&email_digest)?,
    //         org: Uuid::try_parse(row.try_get::<String, _>("org")?)?,
    //         password: safe::VarChar::new(&row.try_get::<String, _>("password")?)?,
    //     })
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grokloc::app::schema;

    #[test]
    fn user_encrypted_test() -> Result<(), Box<dyn Error>> {
        let key = crypt::rand_key();
        let display_name = safe::VarChar::rand();
        let email = safe::VarChar::rand();
        let org = Uuid::new_v4();
        let password = safe::VarChar::new(&crypt::kdf(&crypt::rand_hex(), crypt::MIN_KDF_ROUNDS))?;
        let user = encrypted(&display_name, &email, &org, &password, &key)?;

        let email_digest = crypt::sha256_hex(&email.to_string());
        let iv = crypt::iv(&email_digest);

        let decrypted_api_secret = crypt::decrypt(&key, &iv, &user.api_secret.to_string())?;

        assert_eq!(
            crypt::sha256_hex(&decrypted_api_secret),
            user.api_secret_digest.to_string(),
            "api_secret_digest"
        );

        let decrypted_display_name = crypt::decrypt(&key, &iv, &user.display_name.to_string())?;
        assert_eq!(
            &decrypted_display_name,
            &display_name.to_string(),
            "display name"
        );

        assert_eq!(
            crypt::sha256_hex(&display_name.to_string()),
            user.display_name_digest.to_string(),
            "display_name_digest"
        );

        let decrypted_email = crypt::decrypt(&key, &iv, &user.email.to_string())?;
        assert_eq!(&decrypted_email, &email.to_string(), "email");

        assert_eq!(email_digest, user.email_digest.to_string(), "email_digest");

        assert_eq!(org, user.org, "org");

        assert_eq!(password, user.password, "password");

        Ok(())
    }

    #[tokio::test]
    async fn user_insert_test() -> Result<(), Box<dyn Error>> {
        // build the user
        let key = crypt::rand_key();
        let display_name = safe::VarChar::rand();
        let email = safe::VarChar::rand();
        let org = Uuid::new_v4();
        let password = safe::VarChar::new(&crypt::kdf(&crypt::rand_hex(), crypt::MIN_KDF_ROUNDS))?;
        let user = encrypted(&display_name, &email, &org, &password, &key)?;

        // create the db
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
            .execute(&pool)
            .await?;

        // insert the user
        let mut txn = pool.begin().await?;
        user.insert(&mut txn).await?;
        // implicit rollback

        Ok(())
    }
}
