mod grokloc;
use crate::grokloc::env;

fn main() {
    println!(
        "api version: {}, unit env var: {}",
        grokloc::API_VERSION,
        env::Level::Unit
    );
}
