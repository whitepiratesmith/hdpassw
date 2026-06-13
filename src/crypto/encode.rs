/// Encode a raw 32-byte site key into a human-typeable password.
///
/// Each character is selected by reading two bytes from a BLAKE3 XOF stream
/// (key-derivation mode, context = CTX_ENCODE) and mapping
/// `u16::from_le_bytes(pair) % charset_len` to a character index.
/// Bias is < 0.002% for all supported charsets.
use zeroize::Zeroizing;

use crate::error::{Error, Result};

/// BLAKE3 context for the password-encoding step.
/// Globally unique, hardcoded, never changes for this version.
const CTX_ENCODE: &str = "hdpassw 2026-05-24 encode";

/// Available character sets for generated passwords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Charset {
    /// a-z A-Z (52 chars)
    Alpha,
    /// a-z A-Z 0-9 (62 chars) — default
    #[default]
    Alphanumeric,
    /// a-z A-Z 0-9 + common symbols (95 printable ASCII chars)
    Full,
    /// 0-9 only, for PINs
    Pin,
    /// Lowercase hex, e.g. for API keys
    Hex,
}

impl Charset {
    pub fn chars(self) -> &'static [u8] {
        match self {
            Charset::Alpha => b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
            Charset::Alphanumeric => {
                b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            }
            Charset::Full => {
                b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\
                  0123456789!@#$%^&*()-_=+[]{}|;:,.<>?"
            }
            Charset::Pin => b"0123456789",
            Charset::Hex => b"0123456789abcdef",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "alpha" => Ok(Charset::Alpha),
            "alphanumeric" | "alnum" => Ok(Charset::Alphanumeric),
            "full" => Ok(Charset::Full),
            "pin" => Ok(Charset::Pin),
            "hex" => Ok(Charset::Hex),
            other => Err(Error::InvalidCharset(format!(
                "unknown charset '{other}'; valid: alpha, alphanumeric, full, pin, hex"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Charset::Alpha => "alpha",
            Charset::Alphanumeric => "alphanumeric",
            Charset::Full => "full",
            Charset::Pin => "pin",
            Charset::Hex => "hex",
        }
    }
}

/// Encode `key` into a password of `length` characters drawn from `charset`.
///
/// Uses BLAKE3 in key-derivation mode (XOF output) to expand the 32-byte site
/// key to `length * 2` bytes, then maps each consecutive pair to a character.
pub fn encode(key: &[u8; 32], length: usize, charset: Charset) -> Result<String> {
    if length == 0 || length > 128 {
        return Err(Error::InvalidCharset(format!(
            "length must be 1–128, got {length}"
        )));
    }

    let chars = charset.chars();
    let n = chars.len() as u16;
    let needed = length * 2;

    // Stream `needed` bytes from BLAKE3 in key-derivation mode.
    // The context string provides domain separation from other BLAKE3 uses.
    let mut hasher = blake3::Hasher::new_derive_key(CTX_ENCODE);
    hasher.update(key.as_ref());
    let mut reader = hasher.finalize_xof();
    let mut expanded = Zeroizing::new(vec![0u8; needed]);
    reader.fill(expanded.as_mut_slice());

    let password: String = expanded
        .chunks_exact(2)
        .take(length)
        .map(|pair| {
            let idx = u16::from_le_bytes([pair[0], pair[1]]) % n;
            chars[idx as usize] as char
        })
        .collect();

    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        k[0] = 42;
        k
    }

    #[test]
    fn correct_length() {
        let k = dummy_key();
        for len in [8, 16, 20, 32, 64] {
            let p = encode(&k, len, Charset::Alphanumeric).unwrap();
            assert_eq!(p.len(), len, "len={len}");
        }
    }

    #[test]
    fn only_charset_chars() {
        let k = dummy_key();
        let chars: Vec<char> = Charset::Full.chars().iter().map(|&b| b as char).collect();
        let p = encode(&k, 32, Charset::Full).unwrap();
        for c in p.chars() {
            assert!(chars.contains(&c), "unexpected char: {c}");
        }
    }

    #[test]
    fn deterministic() {
        let k = dummy_key();
        let p1 = encode(&k, 20, Charset::Alphanumeric).unwrap();
        let p2 = encode(&k, 20, Charset::Alphanumeric).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn different_keys_different_passwords() {
        let mut k2 = dummy_key();
        k2[0] = 99;
        let p1 = encode(&dummy_key(), 20, Charset::Alphanumeric).unwrap();
        let p2 = encode(&k2, 20, Charset::Alphanumeric).unwrap();
        assert_ne!(p1, p2);
    }

    #[test]
    fn pin_is_digits_only() {
        let k = dummy_key();
        let p = encode(&k, 6, Charset::Pin).unwrap();
        assert!(p.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn invalid_charset_str() {
        assert!(Charset::from_str("bogus").is_err());
    }
}
