//! org models an orgs row and related db functionality
use crate::grokloc::app::models;
use crate::grokloc::safe;
use uuid;

#[allow(dead_code)]
pub const SCHEMA_VERSION: i8 = 0;

/// Org is the data representation of an orgs row
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Org {
    pub id: uuid::Uuid,
    pub name: safe::VarChar,
    pub owner: uuid::Uuid,
    pub meta: models::Meta,
}
