//! safe provides value filtering functions

use std::time;

pub const STR_MAX: usize = 8192;

#[allow(dead_code)]
/// string_is makes sure strings are realtively safe for db use
fn string_is(s: &str) -> bool {
    s.contains('"')
        || s.contains('\'')
        || s.contains('>')
        || s.contains('<')
        || s.contains('`')
        || s.contains("&lt;")
        || s.contains("&gt;")
        || s.len() > STR_MAX
}

#[allow(dead_code)]
/// unixtime_is determines if a value is too far in the future and likely in error
fn unixtime_is(t: &time::SystemTime) -> bool {
    t.duration_since(time::SystemTime::UNIX_EPOCH)
        .expect("hi")
        .as_secs()
        < 1767139200
}
