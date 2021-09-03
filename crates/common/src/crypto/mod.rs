//! Cryptographic functions that support Yarrbot functions.

use anyhow::{ensure, Context, Result};
use sodiumoxide::{crypto::pwhash::argon2id13, randombytes::randombytes_uniform};
use tokio::task::spawn_blocking;

/// Initializes Sodiumoxide, the cryptography library used in Yarrbot..
pub fn initialize_cryptography() -> Result<()> {
    ensure!(
        sodiumoxide::init().is_ok(),
        "Failed to initialize cryptography library."
    );
    Ok(())
}

/// Hash a given password and return the bytes representing the hash.
pub async fn hash(password: String) -> Result<[u8; 128]> {
    let result = spawn_blocking(move || {
        argon2id13::pwhash(
            password.as_bytes(),
            argon2id13::OPSLIMIT_INTERACTIVE,
            argon2id13::MEMLIMIT_INTERACTIVE,
        )
    })
    .await?;
    ensure!(result.is_ok(), "Failed to hash password.");
    Ok(result.unwrap().0)
}

/// Verify that the given password matches the given hash. Returns
/// true if the passwords match; false otherwise.
///
/// # Remarks
///
/// If the given hash cannot be parsed, this method will return false.
pub async fn verify(password: String, hash: &[u8]) -> bool {
    if let Some(p) = argon2id13::HashedPassword::from_slice(hash) {
        let await_result =
            spawn_blocking(move || argon2id13::pwhash_verify(&p, password.as_bytes())).await;
        await_result.unwrap_or(false)
    } else {
        false
    }
}

/// Generates a random password using characters in the range UTF-8 `U+0021` (exclamation point) to `U+007E` (tilde).
pub fn generate_password(length: Option<u8>) -> Result<String> {
    let l = length.unwrap_or(15);
    let buf: Vec<u8> = (0..l)
        .map(|_| (randombytes_uniform(126 - 33) + 33) as u8)
        .collect();
    String::from_utf8(buf).context("Failed to generate a random password.")
}

#[cfg(test)]
mod tests {
    use crate::crypto::{hash, verify};

    #[tokio::test]
    async fn verify_given_matching_password_returns_true() {
        let password = String::from("I am a password");
        let expected = password.clone();
        let hashed = hash(password).await.unwrap();

        assert!(verify(expected, &hashed).await);
    }

    #[tokio::test]
    async fn verify_given_password_does_not_match_hash_returns_true() {
        let expected = String::from("I am a password");
        let hashed = hash(String::from("But I'm not the same password above"))
            .await
            .unwrap();

        assert!(!verify(expected, &hashed).await);
    }
}
