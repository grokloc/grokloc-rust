//! user models an orgs row and related db functionality
use crate::grokloc::app::models;
use crate::grokloc::crypt;
use crate::grokloc::safe;
use anyhow;
use sqlx;
use sqlx::Row;
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

impl User {
    /// encrypted makes a new User with PII fields encrypted with key as key
    /// and salt derived from the email (which is constrained to be unique in the db)
    ///
    /// if you want a decrypted User, you must read() it from the db
    ///
    /// this is because User's will have referential integrity checks only upon
    /// calling create(), and decrypted Users should always be valid
    #[allow(dead_code)]
    pub fn encrypted(
        display_name: &safe::VarChar,
        email: &safe::VarChar,
        org: &Uuid,
        password: &safe::VarChar, // assumed already derived
        key: &str,
    ) -> Result<User, anyhow::Error> {
        let email_digest = crypt::sha256_hex(&email.to_string());
        let iv = crypt::iv(&email_digest);
        let api_secret_ = Uuid::new_v4();

        Ok(User {
            id: Uuid::new_v4(),
            api_secret: safe::VarChar::new(&crypt::encrypt(key, &iv, &api_secret_.to_string())?)?,
            api_secret_digest: safe::VarChar::new(&crypt::sha256_hex(&api_secret_.to_string()))?,
            display_name: safe::VarChar::new(&crypt::encrypt(
                key,
                &iv,
                &display_name.to_string(),
            )?)?,
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

    /// insert performs db insert with no integrity check on the org (see "create")
    ///
    /// assumed to be called within an existing transaction that includes
    /// consistency checks, so connection handle is a sqlx::Transaction
    #[allow(dead_code)]
    pub async fn insert(
        &self,
        txn: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<(), anyhow::Error> {
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
            return Err(insert_error.into());
        }
        Ok(())
    }

    /// read selects and decrypts a users row to construct a User instance
    #[allow(dead_code)]
    pub async fn read(
        pool: &sqlx::SqlitePool,
        id: &Uuid,
        key: &str,
    ) -> Result<Self, anyhow::Error> {
        let row = sqlx::query(SELECT_QUERY)
            .bind(&id.to_string())
            .fetch_one(pool)
            .await?;
        let email_digest_ = row.try_get::<String, _>("email_digest")?;
        let iv = crypt::iv(&email_digest_);
        let encrypted_api_secret = row.try_get::<String, _>("api_secret")?;
        let api_secret_ = crypt::decrypt(key, &iv, &encrypted_api_secret)?;
        let encrypted_display_name = row.try_get::<String, _>("display_name")?;
        let display_name_ = crypt::decrypt(key, &iv, &encrypted_display_name)?;
        let encrypted_email = row.try_get::<String, _>("email")?;
        let email_ = crypt::decrypt(key, &iv, &encrypted_email)?;

        Ok(Self {
            id: *id,
            api_secret: safe::VarChar::trusted(&api_secret_),
            api_secret_digest: safe::VarChar::trusted(
                &row.try_get::<String, _>("api_secret_digest")?,
            ),
            display_name: safe::VarChar::trusted(&display_name_),
            display_name_digest: safe::VarChar::trusted(
                &row.try_get::<String, _>("display_name_digest")?,
            ),
            email: safe::VarChar::trusted(&email_),
            email_digest: safe::VarChar::trusted(&email_digest_),
            org: Uuid::try_parse(&row.try_get::<String, _>("org")?)?,
            password: safe::VarChar::trusted(&row.try_get::<String, _>("password")?),
            meta: models::Meta::from_db(
                row.try_get::<i64, _>("ctime")?,
                row.try_get::<i64, _>("mtime")?,
                row.try_get::<i8, _>("schema_version")?,
                row.try_get::<i64, _>("status")?,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grokloc::app::schema;

    #[test]
    fn user_encrypted_test() -> Result<(), anyhow::Error> {
        let key = crypt::rand_key();
        let display_name = safe::VarChar::rand();
        let email = safe::VarChar::rand();
        let org = Uuid::new_v4();
        let password = safe::VarChar::new(&crypt::kdf(&crypt::rand_hex(), crypt::MIN_KDF_ROUNDS))?;
        let user = User::encrypted(&display_name, &email, &org, &password, &key)?;

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
    async fn user_insert_test() -> Result<(), anyhow::Error> {
        // build the user
        let key = crypt::rand_key();
        let display_name = safe::VarChar::rand();
        let email = safe::VarChar::rand();
        let org = Uuid::new_v4();
        let password = safe::VarChar::new(&crypt::kdf(&crypt::rand_hex(), crypt::MIN_KDF_ROUNDS))?;
        let user = User::encrypted(&display_name, &email, &org, &password, &key)?;

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
        txn.commit().await?;
        Ok(())
    }

    #[tokio::test]
    async fn user_read_test() -> Result<(), anyhow::Error> {
        // build the user
        let key = crypt::rand_key();
        let display_name = safe::VarChar::rand();
        let email = safe::VarChar::rand();
        let org = Uuid::new_v4();
        let password = safe::VarChar::new(&crypt::kdf(&crypt::rand_hex(), crypt::MIN_KDF_ROUNDS))?;
        let user = User::encrypted(&display_name, &email, &org, &password, &key)?;

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
        txn.commit().await?;

        // read that user
        let user_read = match User::read(&pool, &user.id, &key).await {
            Err(_) => unreachable!(),
            Ok(v) => v,
        };

        assert_eq!(user.id, user_read.id);

        // user.api_secret is encrypted
        assert_ne!(user.api_secret, user_read.api_secret);
        assert_eq!(
            crypt::sha256_hex(&user_read.api_secret.to_string()),
            user_read.api_secret_digest.to_string()
        );

        // user.display_name is encrypted
        assert_ne!(user.display_name, user_read.display_name);
        assert_eq!(user.display_name_digest, user_read.display_name_digest);
        assert_eq!(
            crypt::sha256_hex(&user_read.display_name.to_string()),
            user_read.display_name_digest.to_string()
        );

        // user.email is encrypted
        assert_ne!(user.email, user_read.email);
        assert_eq!(user.email_digest, user_read.email_digest);
        assert_eq!(
            crypt::sha256_hex(&user_read.email.to_string()),
            user_read.email_digest.to_string()
        );

        assert_eq!(user.org, user_read.org);
        assert_eq!(user.password, user_read.password);

        assert_eq!(user.meta.schema_version, user_read.meta.schema_version);
        assert_eq!(user.meta.status, user_read.meta.status);

        assert!(user.meta.ctime < user_read.meta.ctime);
        assert!(user.meta.mtime < user_read.meta.mtime);

        Ok(())
    }

    #[tokio::test]
    async fn user_read_miss_test() -> Result<(), anyhow::Error> {
        // create the db
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
            .execute(&pool)
            .await?;
        let user_read_result = match User::read(&pool, &Uuid::new_v4(), &crypt::rand_key()).await {
            Err(e) => e,
            Ok(_) => unreachable!(),
        };
        assert!(matches!(
            user_read_result.downcast_ref::<sqlx::Error>(),
            Some(sqlx::Error::RowNotFound)
        ));
        Ok(())
    }
}
