//! org models an orgs row and related db functionality
use crate::grokloc::app::admin::user::User;
use crate::grokloc::app::models;
use crate::grokloc::safe;
use anyhow;
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
(?,?,?,?,?,?,?,?,?,?,?)
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

        if let Err(insert_error) = sqlx::query(INSERT_QUERY)
            .bind(org.id.to_string())
            .bind(org.name.to_string())
            .bind(org.owner.to_string())
            .bind(org.meta.status.to_int())
            .bind(org.meta.schema_version)
            .execute(&mut txn)
            .await
        {
            return Err(insert_error.into());
        }

        txn.commit().await?;

        Ok((org, owner))
    }
}
