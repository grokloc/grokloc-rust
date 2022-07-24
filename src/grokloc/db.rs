//! db contains functions and symbols for db-related errors
use anyhow;
use sqlx;
use thiserror::Error;

/// Err covers potential error state arising from db operations
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum Err {
    #[error("org constraint violation")]
    OrgViolation,
    #[error("user constraint violation")]
    UserViolation,
    #[error("bad row values")]
    BadRowValues,
}

/// sqlx_duplicate should match unique constraints for sqlite and pg
#[allow(dead_code)]
pub fn sqlx_duplicate(error: &sqlx::Error) -> bool {
    let mut s = error.to_string();
    s.make_ascii_lowercase();
    s.contains("unique constraint")
}

/// anyhow_sqlx_duplicate should match unique constraints for sqlite and pg,
/// downcasting from anyhow::Error
#[allow(dead_code)]
pub fn anyhow_sqlx_duplicate(error: &anyhow::Error) -> bool {
    match error.downcast_ref::<sqlx::Error>() {
        None => false,
        Some(e) => sqlx_duplicate(e),
    }
}

/// sqlx_row_not_found returns true if error is sqlx::Error::RowNotFound
#[allow(dead_code)]
pub fn sqlx_row_not_found(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::RowNotFound)
}

/// anyhow_sqlx_row_not_found returns true if error is sqlx::Error::RowNotFound
/// downcasting from anyhow::Error
#[allow(dead_code)]
pub fn anyhow_sqlx_row_not_found(error: &anyhow::Error) -> bool {
    matches!(
        error.downcast_ref::<sqlx::Error>(),
        Some(sqlx::Error::RowNotFound)
    )
}
