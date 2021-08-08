//! Cryptographic functions that support Yarrbot functions.

use anyhow::{ensure, Error};
use sodiumoxide::crypto::pwhash::argon2id13;

/// Initializes Sodiumoxide.
fn initialize_sodiumoxide_or_err() -> Result<(), Error> {
    ensure!(
        sodiumoxide::init().is_ok(),
        "Failed to initialize cryptography library."
    );
    Ok(())
}

/// Hash a given password and return the bytes representing the hash.
pub fn hash(password: &str) -> Result<[u8; 128], Error> {
    initialize_sodiumoxide_or_err()?;
    let result = argon2id13::pwhash(
        password.as_bytes(),
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE,
    );
    ensure!(result.is_ok(), "Failed to hash password.");
    Ok(result.unwrap().0)
}

/// Verify that the given password matches the given hash. Returns
/// true if the passwords match; false otherwise.
///
/// # Remarks
///
/// If the given hash cannot be parsed, this method will return false.
pub fn verify(password: &str, hash: &[u8]) -> bool {
    match initialize_sodiumoxide_or_err() {
        Ok(_) => (),
        _ => return false,
    };
    match argon2id13::HashedPassword::from_slice(hash) {
        Some(h) => argon2id13::pwhash_verify(&h, password.as_bytes()),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{hash, verify};

    #[test]
    fn verify_given_matching_password_returns_true() {
        let password = "I am a password";
        let hashed = hash(password).unwrap();

        assert!(verify(password, &hashed));
    }

    #[test]
    fn verify_given_password_does_not_match_hash_returns_true() {
        let password = "I am a password";
        let hashed = hash("But I'm not the same password above").unwrap();

        assert!(!verify(password, &hashed));
    }
}
