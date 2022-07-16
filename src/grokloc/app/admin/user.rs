//! user models an orgs row and related db functionality
use crate::grokloc::app::models;
use crate::grokloc::crypt;
use crate::grokloc::safe;
use std::error::Error;
use uuid::Uuid;

pub const SCHEMA_VERSION: i8 = 0;

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
    // per-user iv is derived from the email address as follows:
    let email_digest = crypt::sha256_hex(&email.to_string()); // as String
    let iv = crypt::iv_truncate(&email_digest);
    let api_secret = safe::VarChar::new(&Uuid::new_v4().to_string())?;
    Ok(User {
        id: Uuid::new_v4(),
        api_secret: safe::VarChar::new(&crypt::encrypt(key, &iv, &api_secret.to_string())?)?,
        api_secret_digest: safe::VarChar::new(&crypt::sha256_hex(&api_secret.to_string()))?,
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
