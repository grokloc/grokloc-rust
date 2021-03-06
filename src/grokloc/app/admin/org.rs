//! org models an orgs row and related db functionality
use crate::grokloc::app::admin::user::User;
use crate::grokloc::app::models;
use crate::grokloc::safe;
use anyhow;
use sqlx;
use sqlx::Row;
use uuid::Uuid;

pub const SCHEMA_VERSION: i8 = 0;

pub const INSERT_QUERY: &str = r#"
insert into orgs
(id,
 name,
 owner,
 schema_version,
 status)
 values
(?,?,?,?,?)
"#;

pub const SELECT_QUERY: &str = r#"
select
 name,
 owner,
 ctime,
 mtime,
 schema_version,
 status
from orgs
where id = ?
"#;

pub const UPDATE_STATUS_QUERY: &str = r#"
update orgs set status = ? where id = ?;
"#;

/// Org is the data representation of an orgs row
#[derive(Clone, Debug)]
pub struct Org {
    pub id: uuid::Uuid,
    pub name: safe::VarChar,
    pub owner: uuid::Uuid,
    pub meta: models::Meta,
}

impl Org {
    /// insert performs db insert with no integrity check on the owner (see create)
    #[allow(dead_code)]
    pub async fn insert(
        &self,
        txn: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<(), anyhow::Error> {
        if let Err(insert_error) = sqlx::query(INSERT_QUERY)
            .bind(self.id.to_string())
            .bind(self.name.to_string())
            .bind(self.owner.to_string())
            .bind(self.meta.schema_version)
            .bind(self.meta.status.to_int())
            .execute(txn)
            .await
        {
            return Err(insert_error.into());
        }

        Ok(())
    }

    /// create forms a new Org with a new User as owner
    pub async fn create(
        pool: &sqlx::SqlitePool,
        name: &safe::VarChar,
        owner_display_name: &safe::VarChar,
        owner_email: &safe::VarChar,
        owner_password: &safe::VarChar,
        key: &str,
    ) -> Result<(Self, User), anyhow::Error> {
        let id = Uuid::new_v4();

        // build and insert org owner
        let mut owner = User::encrypted(owner_display_name, owner_email, &id, owner_password, key)?;
        owner.meta.status = models::Status::Active;
        let mut txn = pool.begin().await?;
        owner.insert(&mut txn).await?;

        let org = Self {
            id,
            name: name.clone(),
            owner: owner.id,
            meta: models::Meta {
                status: models::Status::Active,
                schema_version: SCHEMA_VERSION,
                ..Default::default()
            },
        };

        org.insert(&mut txn).await?;

        txn.commit().await?;

        Ok((org, owner))
    }

    /// read selects a row an orgs row to construct an Org instance
    #[allow(dead_code)]
    pub async fn read(pool: &sqlx::SqlitePool, id: &Uuid) -> Result<Self, anyhow::Error> {
        let row = sqlx::query(SELECT_QUERY)
            .bind(&id.to_string())
            .fetch_one(pool)
            .await?;

        Ok(Self {
            id: *id,
            name: safe::VarChar::trusted(&row.try_get::<String, _>("name")?),
            owner: Uuid::try_parse(&row.try_get::<String, _>("owner")?)?,
            meta: models::Meta::from_db(
                row.try_get::<i64, _>("ctime")?,
                row.try_get::<i64, _>("mtime")?,
                row.try_get::<i8, _>("schema_version")?,
                row.try_get::<i64, _>("status")?,
            )?,
        })
    }

    /// update_status updates the org status
    #[allow(dead_code)]
    pub async fn update_status(
        &mut self,
        pool: &sqlx::SqlitePool,
        new_status: models::Status,
    ) -> Result<(), anyhow::Error> {
        let update_result = match sqlx::query(UPDATE_STATUS_QUERY)
            .bind(new_status.to_int())
            .bind(self.id.to_string())
            .execute(pool)
            .await
        {
            Err(e) => return Err(e.into()),
            Ok(v) => v,
        };

        if update_result.rows_affected() != 1 {
            return Err(sqlx::Error::RowNotFound.into());
        }

        // the update to the db was a success, set the internal field
        self.meta.status = new_status;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grokloc::app::admin::user::User;
    use crate::grokloc::app::schema;
    use crate::grokloc::crypt;
    use crate::grokloc::db;
    use anyhow;

    #[tokio::test]
    async fn org_insert_test() -> Result<(), anyhow::Error> {
        let org = Org {
            id: Uuid::new_v4(),
            name: safe::VarChar::rand(),
            owner: Uuid::new_v4(),
            meta: models::Meta {
                schema_version: SCHEMA_VERSION,
                ..Default::default()
            },
        };

        // create the db
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
            .execute(&pool)
            .await?;

        // insert the org
        let mut txn = pool.begin().await?;
        org.insert(&mut txn).await?;
        // implicit rollback
        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn org_read_test() -> Result<(), anyhow::Error> {
        let org = Org {
            id: Uuid::new_v4(),
            name: safe::VarChar::rand(),
            owner: Uuid::new_v4(),
            meta: models::Meta {
                schema_version: SCHEMA_VERSION,
                ..Default::default()
            },
        };

        // create the db
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
            .execute(&pool)
            .await?;

        // insert the org
        let mut txn = pool.begin().await?;
        org.insert(&mut txn).await?;
        // implicit rollback
        txn.commit().await?;

        // read that org
        let org_read = match Org::read(&pool, &org.id).await {
            Err(_) => unreachable!(),
            Ok(v) => v,
        };

        assert_eq!(org.id, org_read.id);
        assert_eq!(org.name, org_read.name);
        assert_eq!(org.owner, org_read.owner);
        assert!(org.meta.ctime < org_read.meta.ctime);
        assert!(org.meta.mtime < org_read.meta.mtime);

        Ok(())
    }

    #[tokio::test]
    async fn org_read_miss_test() -> Result<(), anyhow::Error> {
        // create the db
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
            .execute(&pool)
            .await?;
        let org_read_result = match Org::read(&pool, &Uuid::new_v4()).await {
            Err(e) => e,
            Ok(_) => unreachable!(),
        };

        assert!(db::anyhow_sqlx_row_not_found(&org_read_result));

        Ok(())
    }

    #[tokio::test]
    async fn org_create_test() -> Result<(), anyhow::Error> {
        // create the db
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
            .execute(&pool)
            .await?;

        let key = crypt::rand_key();
        let name = safe::VarChar::rand();
        let owner_display_name = safe::VarChar::rand();
        let owner_email = safe::VarChar::rand();
        let owner_password =
            safe::VarChar::new(&crypt::kdf(&crypt::rand_hex(), crypt::MIN_KDF_ROUNDS))?;

        let (org, owner) = Org::create(
            &pool,
            &name,
            &owner_display_name,
            &owner_email,
            &owner_password,
            &key,
        )
        .await?;

        // read the org
        let org_read = match Org::read(&pool, &org.id).await {
            Err(_) => unreachable!(),
            Ok(v) => v,
        };

        assert_eq!(org.id, org_read.id);
        assert!(org.meta.ctime < org_read.meta.ctime);
        assert!(org.meta.mtime < org_read.meta.mtime);

        // read the owner
        let user_read = match User::read(&pool, &owner.id, &key).await {
            Err(_) => unreachable!(),
            Ok(v) => v,
        };

        assert_eq!(owner.id, user_read.id);
        assert_eq!(models::Status::Active, user_read.meta.status);
        assert!(owner.meta.ctime < user_read.meta.ctime);
        assert!(owner.meta.mtime < user_read.meta.mtime);

        Ok(())
    }

    #[tokio::test]
    async fn org_update_status_test() -> Result<(), anyhow::Error> {
        // create the db
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
            .execute(&pool)
            .await?;

        let key = crypt::rand_key();
        let name = safe::VarChar::rand();
        let owner_display_name = safe::VarChar::rand();
        let owner_email = safe::VarChar::rand();
        let owner_password =
            safe::VarChar::new(&crypt::kdf(&crypt::rand_hex(), crypt::MIN_KDF_ROUNDS))?;

        let (mut org, _) = Org::create(
            &pool,
            &name,
            &owner_display_name,
            &owner_email,
            &owner_password,
            &key,
        )
        .await?;

        // update the status of the org
        org.update_status(&pool, models::Status::Inactive).await?;

        // read the org
        let org_read = match Org::read(&pool, &org.id).await {
            Err(_) => unreachable!(),
            Ok(v) => v,
        };

        assert_eq!(org.id, org_read.id);
        assert_eq!(models::Status::Inactive, org_read.meta.status);
        assert!(org.meta.ctime < org_read.meta.ctime);
        assert!(org.meta.mtime < org_read.meta.mtime);

        Ok(())
    }

    #[tokio::test]
    async fn org_update_status_miss_test() -> Result<(), anyhow::Error> {
        // create the db
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;
        sqlx::query(schema::APP_CREATE_SCHEMA_SQLITE)
            .execute(&pool)
            .await?;

        // new org is never inserted
        let mut org = Org {
            id: Uuid::new_v4(),
            name: safe::VarChar::rand(),
            owner: Uuid::new_v4(),
            meta: models::Meta {
                status: models::Status::Active,
                schema_version: SCHEMA_VERSION,
                ..Default::default()
            },
        };

        match org.update_status(&pool, models::Status::Active).await {
            Ok(_) => unreachable!(),
            Err(e) => assert!(db::anyhow_sqlx_row_not_found(&e)),
        };

        Ok(())
    }
}
