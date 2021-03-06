//! safe provides value filtering functions
use crate::grokloc::crypt;
use regex::Regex;
use std::fmt;
use thiserror::Error;

pub const STR_MAX: usize = 8192;

/// Err abstracts over safe-value error types
#[derive(Copy, Clone, Debug, Error, PartialEq)]
pub enum Err {
    #[error("unsafe string")]
    UnsafeString,
}

/// string_ok makes sure strings are realtively safe for db use
pub fn string_ok(s: &str) -> bool {
    let re_insert = Regex::new(r"insert\s+into").unwrap();
    let re_table = Regex::new(r"(?:drop|create)\s+(table|database)").unwrap();
    let re_query = Regex::new(r"(?:select|update)\s+").unwrap();
    let lc_s = s.to_lowercase();
    !(s.contains('"')
        || s.contains('\'')
        || s.contains('>')
        || s.contains('<')
        || s.contains('`')
        || lc_s.contains("&lt;")
        || lc_s.contains("&gt;")
        || re_insert.is_match(&lc_s)
        || re_table.is_match(&lc_s)
        || re_query.is_match(&lc_s)
        || s.len() > STR_MAX
        || s.is_empty())
}

/// VarChar is a string container that proves that the value is safe for db storage
#[derive(Clone, Debug, PartialEq)]
pub struct VarChar(String);

impl VarChar {
    /// new produces a VarChar iff the input string is safe
    pub fn new(raw: &str) -> Result<VarChar, Err> {
        match string_ok(raw) {
            true => Ok(VarChar(raw.to_string())),
            false => Err(Err::UnsafeString),
        }
    }

    /// trusted can be used to construct a VarChar if the input is trustworthy
    pub fn trusted(raw: &str) -> VarChar {
        VarChar(raw.to_string())
    }

    /// rand produces a (long!) random string from a uuid
    pub fn rand() -> VarChar {
        VarChar(crypt::rand_hex())
    }
}

impl fmt::Display for VarChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_ok_test() {
        assert!(!string_ok(""));
        assert!(!string_ok("hello ' there"));
        assert!(!string_ok("hello > there"));
        assert!(!string_ok("hello < there"));
        assert!(!string_ok("select col from table"));
        assert!(!string_ok("update col"));
        assert!(!string_ok("hello drop table"));
        assert!(!string_ok("hello create table"));
    }

    #[test]
    fn varchar_ok_test() -> Result<(), Err> {
        assert_eq!(VarChar::new("ok")?.to_string(), "ok");
        Ok(())
    }

    #[test]
    fn varchar_err_test() -> Result<(), Err> {
        let vc = VarChar::new("select col from table");
        assert!(matches!(vc, Err(Err::UnsafeString)));
        Ok(())
    }

    #[test]
    fn clone_test() -> Result<(), Err> {
        let vc0 = VarChar::new("ok");
        let vc1 = vc0.clone();
        assert_eq!(vc0, vc1);
        Ok(())
    }
}
