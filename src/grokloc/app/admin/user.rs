//! user models an orgs row and related db functionality
use crate::grokloc::app::models;
use crate::grokloc::safe;
use std::err::Error;
use uuid;

pub const SCHEMA_VERSION: i8 = 0;

/// User is the data representation of an users row
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct User {
    pub id: uuid::Uuid,
    pub api_secret: uuid::Uuid,
    pub api_secret_digest: safe::VarChar,
    pub display_name: safe::VarChar,
    pub display_name_digest: safe::VarChar,
    pub email: safe::VarChar,
    pub email_digest: safe::VarChar,
    pub org: uuid::Uuid,
    pub password: safe::VarChar,
    pub meta: models::Meta,
}

// pub fn encrypted(
//     display_name: String,
//     email: String,
//     org: uuid::Uuid,
//     password: String,
//     key: String,
// ) -> Result<User, Error> {}
