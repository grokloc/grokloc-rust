//! crypt provides functions and symbols for common encryption patterns
use bcrypt;
use hex;
use openssl::rand::rand_bytes;
use openssl::sha::sha256;
use openssl::symm::decrypt as openssl_decrypt;
use openssl::symm::encrypt as openssl_encrypt;
use openssl::symm::Cipher;
use std::str;
use thiserror::Error;

pub const KEY_LEN: usize = 32;
pub const IV_LEN: usize = 32;

#[allow(dead_code)]
pub const MIN_KDF_ROUNDS: u32 = 4;
#[allow(dead_code)]
pub const DEFAULT_KDF_ROUNDS: u32 = bcrypt::DEFAULT_COST;
#[allow(dead_code)]
pub const MAX_KDF_ROUNDS: u32 = 31;

/// Err indicates a malformed key or nonce
#[derive(Debug, Error, PartialEq)]
pub enum Err {
    #[error("bad key length")]
    KeyLength,
    #[error("bad iv length")]
    IVLength,
    #[error("cipher error: {0}")]
    Cipher(String),
}

/// decrypt produces a cleartext message using:
/// key: a hex-encoded str of len KEY_LEN
/// iv: a hex-encoded str of len IV_LEN
/// c: a hex-encoded str produced by encrypt(...)
#[allow(dead_code)]
pub fn decrypt(key: &str, iv: &str, c: &str) -> Result<String, Err> {
    if key.len() != KEY_LEN {
        return Err(Err::KeyLength);
    }
    let key_decoded = match hex::decode(key) {
        Ok(bs) => bs,
        // we are using a corrupt key
        Err(error) => panic!("key hex decode: {:?}", error),
    };
    let key_slice = &key_decoded[..];
    if iv.len() != IV_LEN {
        return Err(Err::IVLength);
    }
    let iv_decoded = match hex::decode(iv) {
        Ok(bs) => bs,
        // we are using a corrupt iv
        Err(error) => panic!("iv hex decode: {:?}", error),
    };
    let iv_slice = &iv_decoded[..];
    let c_decoded = match hex::decode(c) {
        Ok(bs) => bs,
        // ciphertext produced by encrypt(...)
        Err(error) => panic!("ciphertext hex decode: {:?}", error),
    };
    let cipher = Cipher::aes_128_cbc();
    let c_bs = &c_decoded[..];
    let decrypt_result = openssl_decrypt(cipher, key_slice, Some(iv_slice), c_bs);
    match decrypt_result {
        Ok(m) => Ok(String::from_utf8(m).unwrap()),
        Err(error) => Err(Err::Cipher(format!("{:?}", error))),
    }
}

/// encrypt produces a hex-encoded ciphertext using
/// key: a hex-encoded str of len KEY_LEN
/// iv: a hex-encoded str of len IV_LEN
/// m: plaintext message
#[allow(dead_code)]
pub fn encrypt(key: &str, iv: &str, m: &str) -> Result<String, Err> {
    if key.len() != KEY_LEN {
        return Err(Err::KeyLength);
    }
    let key_decoded = match hex::decode(key) {
        Ok(bs) => bs,
        // we are using a corrupt key
        Err(error) => panic!("key hex decode: {:?}", error),
    };
    let key_slice = &key_decoded[..];
    if iv.len() != IV_LEN {
        return Err(Err::IVLength);
    }
    let iv_decoded = match hex::decode(iv) {
        Ok(bs) => bs,
        // we are using a corrupt iv
        Err(error) => panic!("iv hex decode: {:?}", error),
    };
    let iv_slice = &iv_decoded[..];
    let cipher = Cipher::aes_128_cbc();
    let encrypt_result = openssl_encrypt(cipher, key_slice, Some(iv_slice), m.as_bytes());
    match encrypt_result {
        Ok(c) => Ok(hex::encode(c)),
        Err(error) => Err(Err::Cipher(format!("{:?}", error))),
    }
}

/// iv_truncate truncates an existing salt seed string to IV_LEN
#[allow(dead_code)]
pub fn iv_truncate(s: &str) -> String {
    let mut v = String::from(s);
    v.truncate(IV_LEN);
    v
}

/// iv constructs a per-input salt String (len: IV_LEN)
#[allow(dead_code)]
pub fn iv(s: &str) -> String {
    iv_truncate(&sha256_hex(s))
}

/// kdf creates a safe-to-store password derivation
#[allow(dead_code)]
pub fn kdf(s: &str, cost: u32) -> String {
    bcrypt::hash(s, cost).unwrap()
}

/// kdf_verify returns true if s matches the password that formed hashed
#[allow(dead_code)]
pub fn kdf_verify(s: &str, hashed: &str) -> bool {
    bcrypt::verify(s, hashed).unwrap()
}

/// rand_hex returns a new random hex String (len: 64)
#[allow(dead_code)]
pub fn rand_hex() -> String {
    let mut buf = [0; 32];
    rand_bytes(&mut buf).unwrap();
    hex::encode(buf)
}

/// rand_key returns a new random encryption key (len: KEY_LEN)
#[allow(dead_code)]
pub fn rand_key() -> String {
    let mut buf = [0; KEY_LEN / 2];
    rand_bytes(&mut buf).unwrap();
    hex::encode(buf)
}

/// rand_iv returns a new random encryption iv (len: IV_LEN)
#[allow(dead_code)]
pub fn rand_iv() -> String {
    let mut buf = [0; IV_LEN / 2];
    rand_bytes(&mut buf).unwrap();
    hex::encode(buf)
}

/// sha256_hex returns the hex-encoded String of the sha256 digest of s
#[allow(dead_code)]
pub fn sha256_hex(s: &str) -> String {
    hex::encode(sha256(s.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypt_test_encrypt_decrypt() {
        let key = rand_key();
        let iv = rand_iv();
        let o = "abc";
        let c_result = encrypt(&key, &iv, o);
        assert!(c_result.is_ok(), "encrypt ok");
        let c = c_result.unwrap();
        assert!(!c.is_empty(), "encrypt");

        // decrypt using key, iv
        let m_result = decrypt(&key, &iv, &c);
        assert!(m_result.is_ok(), "decrypt ok");
        let m = m_result.unwrap();
        assert_eq!(o, m, "round trip");

        // try decrypt with different key
        let m_result_bad_key = decrypt(&rand_key(), &iv, &c);
        assert!(m_result_bad_key.is_err(), "decrypt bad key caught");

        // try decrypt with different iv
        let m_result_bad_iv = decrypt(&key, &rand_iv(), &c);
        assert!(m_result_bad_iv.is_err(), "decrypt bad iv caught");

        // try encrypt with a bad len key
        let c_result_bad_key_len = encrypt(&rand_hex(), &iv, o);
        assert!(c_result_bad_key_len.is_err(), "encrypt bad key len caught");

        // try encrypt with a bad len iv
        let c_result_bad_iv_len = encrypt(&key, &rand_hex(), o);
        assert!(c_result_bad_iv_len.is_err(), "encrypt bad iv len caught");

        // try decrypt with a bad len key
        let m_result_bad_key_len = decrypt(&rand_hex(), &iv, &c);
        assert!(m_result_bad_key_len.is_err(), "decrypt bad key len caught");

        // try decrypt with a bad len iv
        let m_result_bad_iv_len = decrypt(&key, &rand_hex(), &c);
        assert!(m_result_bad_iv_len.is_err(), "decrypt bad iv len caught");

        // try decrypt with a bad len key
        let m_result_bad_key_len = decrypt(&rand_hex(), &iv, &c);
        assert!(m_result_bad_key_len.is_err(), "decrypt bad key len caught");

        // try decrypt with a bad len iv
        let m_result_bad_iv_len = decrypt(&key, &rand_hex(), &c);
        assert!(m_result_bad_iv_len.is_err(), "decrypt bad iv len caught");
    }

    #[test]
    fn crypt_test_iv() {
        assert!(iv("abc").len() == IV_LEN, "iv");
    }

    #[test]
    fn crypt_test_kdf() {
        let hashed = kdf("abc", MIN_KDF_ROUNDS);
        assert!(kdf_verify("abc", &hashed), "password match");
        assert!(!kdf_verify("def", &hashed), "password mismatch");
    }

    #[test]
    fn crypt_test_rand_hex() {
        assert!(rand_hex().len() == 64, "rand_hex");
    }

    #[test]
    fn crypt_test_rand_key() {
        assert!(rand_key().len() == KEY_LEN, "rand_key");
    }

    #[test]
    fn crypt_test_rand_nonce() {
        assert!(rand_iv().len() == IV_LEN, "rand_iv");
    }

    #[test]
    fn crypt_test_sha256_hex() {
        assert_eq!(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            sha256_hex("abc"),
            "sha256_hex"
        );
    }
}
