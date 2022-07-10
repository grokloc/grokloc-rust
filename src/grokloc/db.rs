//! db contains functions and symbols for db-related errors
use sqlx;
use std::fmt;

/// Err covers potential error state arising from db operations
#[derive(Debug)]
#[allow(dead_code)]
pub enum Err {
    OrgViolation,
    UserViolation,
    UniquenessViolation,
    BadRowValues,
    SQLx(sqlx::Error),
}

impl Err {
    #[allow(dead_code)]
    pub fn is_sqlx_duplicate(&self) -> bool {
        match self {
            Self::SQLx(error) => sqlx_duplicate(error),
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn is_sqlx_row_not_found(&self) -> bool {
        matches!(self, Self::SQLx(sqlx::Error::RowNotFound))
    }
}

impl fmt::Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // {:?} -> just use debug formatter generated by #[derive(Debug)]
        write!(f, "{:?}", self)
    }
}

/// sqlx_duplicate should match unique constraints for sqlite and pg
#[allow(dead_code)]
pub fn sqlx_duplicate(error: &sqlx::Error) -> bool {
    let mut s = error.to_string();
    s.make_ascii_lowercase();
    s.contains("unique constraint")
}
