//! org models an orgs row and related db functionality
use crate::grokloc::app::models;
use crate::grokloc::safe;
use uuid;

pub const SCHEMA_VERSION: i8 = 0;

/// Instance is the data representation of an orgs row
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct Instance {
    pub id: uuid::Uuid,
    pub name: safe::VarChar,
    pub owner: uuid::Uuid,
    pub meta: models::Meta,
}