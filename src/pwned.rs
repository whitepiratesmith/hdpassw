//! Checks a password against the "Have I Been Pwned" Pwned Passwords range
//! API (<https://haveibeenpwned.com/API/v3#PwnedPasswords>).
//!
//! Uses k-anonymity: only the first 5 hex characters of the password's
//! SHA-1 hash are sent over the network. The full hash, and the password
//! itself, never leave the machine. No API key is required for this
//! endpoint.

use sha1::{Digest, Sha1};

use crate::error::{Error, Result};

/// Outcome of a pwned-password lookup.
pub enum PwnedStatus {
    /// Hash prefix matched no entry in the breach corpus.
    Safe,
    /// Hash matched a known breached password, seen `count` times.
    Pwned(u64),
}

/// Look up `password` against the Pwned Passwords range API.
pub fn check(password: &str) -> Result<PwnedStatus> {
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    let hex_hash = hex::encode_upper(hasher.finalize());
    let (prefix, suffix) = hex_hash.split_at(5);

    let url = format!("https://api.pwnedpasswords.com/range/{prefix}");
    let body: String = ureq::get(&url)
        .call()
        .map_err(|e| Error::Pwned(e.to_string()))?
        .into_string()
        .map_err(|e| Error::Pwned(e.to_string()))?;

    for line in body.lines() {
        if let Some((line_suffix, count)) = line.split_once(':')
            && line_suffix.eq_ignore_ascii_case(suffix)
        {
            let count = count.trim().parse().unwrap_or(0);
            return Ok(PwnedStatus::Pwned(count));
        }
    }

    Ok(PwnedStatus::Safe)
}
