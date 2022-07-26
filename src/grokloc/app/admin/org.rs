//! org models an orgs row and related db functionality
use crate::grokloc::app::admin::user::User;
use crate::grokloc::app::models;
use crate::grokloc::safe;
use anyhow;
use sqlx;
use uuid::Uuid;

#[allow(dead_code)]
pub const SCHEMA_VERSION: i8 = 0;

pub const INSERT_QUERY: &str = r#"
insert into orgs
(id,
 name,
 owner,
 status,
 schema_version)
 values
(?,?,?,?,?)
"#;

/// Org is the data representation of an orgs row
#[derive(Clone, Debug)]
#[allow(dead_code)]
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
            .bind(self.meta.status.to_int())
            .bind(self.meta.schema_version)
            .execute(txn)
            .await
        {
            return Err(insert_error.into());
        }

        Ok(())
    }

    /// create forms a new Org with a new User as owner
    #[allow(dead_code)]
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
        let owner = User::encrypted(owner_display_name, owner_email, &id, owner_password, key)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grokloc::app::schema;
    use anyhow;

    #[tokio::test]
    async fn org_create_test() -> Result<(), anyhow::Error> {
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
}
