//! models contains cross-model definitions
use crate::grokloc::db;
use chrono;
use std::{default, fmt};
use thiserror::Error;

/// Err covers various generic model errors
#[derive(Debug, Error, PartialEq)]
pub enum Err {
    #[error("unknown status")]
    UnknownStatus,
}

/// Status describes model status
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Unconfirmed,
    Active,
    Inactive,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // {:?} -> just use debug formatter generated by #[derive(Debug)]
        write!(f, "{:?}", self)
    }
}

impl Status {
    /// translate a Status to its database representation
    #[allow(dead_code)]
    fn to_int(self) -> i64 {
        match self {
            Status::Unconfirmed => 1,
            Status::Active => 2,
            Status::Inactive => 3,
        }
    }

    /// translate a Status from its database representation
    #[allow(dead_code)]
    fn from_int(i: i64) -> Result<Self, Err> {
        match i {
            1 => Ok(Status::Unconfirmed),
            2 => Ok(Status::Active),
            3 => Ok(Status::Inactive),
            _ => Err(Err::UnknownStatus),
        }
    }
}

/// Meta contains key model metadata fields shared by all table models
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct Meta {
    pub ctime: chrono::DateTime<chrono::Utc>,
    pub mtime: chrono::DateTime<chrono::Utc>,
    pub schema_version: i8,
    pub status: Status,
}

impl default::Default for Meta {
    fn default() -> Self {
        Meta {
            ctime: chrono::DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp(0, 0),
                chrono::Utc,
            ),
            mtime: chrono::DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp(0, 0),
                chrono::Utc,
            ),
            schema_version: 0,
            status: Status::Unconfirmed,
        }
    }
}

impl Meta {
    #[allow(dead_code)]
    pub fn from_row_vals(
        ctime: i64,
        mtime: i64,
        schema_version: i8,
        status: Status,
    ) -> Result<Meta, db::Err> {
        Ok(Meta {
            ctime: chrono::DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp(ctime, 0),
                chrono::Utc,
            ),
            mtime: chrono::DateTime::from_utc(
                chrono::NaiveDateTime::from_timestamp(mtime, 0),
                chrono::Utc,
            ),
            schema_version,
            status,
        })
    }
}
