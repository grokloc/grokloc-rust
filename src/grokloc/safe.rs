//! safe provides value filtering functions

use regex::Regex;
use std::time;

pub const STR_MAX: usize = 8192;

pub const UNIXTIME_MAX: u64 = 1767139200;

#[allow(dead_code)]
/// string_is makes sure strings are realtively safe for db use
fn string_is(s: &str) -> bool {
    let re_insert = Regex::new(r"insert\s+into").unwrap();
    let re_table = Regex::new(r"(?:drop|create)\s+table").unwrap();
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

#[allow(dead_code)]
/// unixtime_is determines if a value is too far in the future and likely in error
fn unixtime_is(t: &time::SystemTime) -> bool {
    t.duration_since(time::SystemTime::UNIX_EPOCH)
        .expect("duration epoch failure")
        .as_secs()
        < UNIXTIME_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_is_test() {
        assert!(!string_is(""));
        assert!(!string_is("hello ' there"));
        assert!(!string_is("hello > there"));
        assert!(!string_is("hello < there"));
        assert!(!string_is("select col from table"));
        assert!(!string_is("update col"));
        assert!(!string_is("hello drop table"));
        assert!(!string_is("hello create table"));
    }
}
