/// BIP39 mnemonic handling.
///
/// Seed derivation follows the BIP39 spec exactly:
///   PBKDF2-HMAC-SHA512(password=mnemonic_sentence, salt="mnemonic"+passphrase,
///                       iterations=2048, dklen=64)
use bip39::Mnemonic;
use rand::RngCore;
use zeroize::Zeroizing;

use crate::error::{Error, Result};

/// Generate a new random 24-word BIP39 mnemonic (256 bits of entropy).
pub fn generate() -> Mnemonic {
    let mut entropy = [0u8; 32]; // 32 bytes = 256 bits = 24 words
    rand::thread_rng().fill_bytes(&mut entropy);
    Mnemonic::from_entropy(&entropy).expect("32 bytes is always valid BIP39 entropy")
}

/// Parse and validate an existing mnemonic phrase.
pub fn parse(phrase: &str) -> Result<Mnemonic> {
    Mnemonic::parse(phrase).map_err(|e| Error::InvalidMnemonic(e.to_string()))
}

/// Derive the 64-byte BIP39 seed from a mnemonic and optional passphrase.
/// The passphrase is an additional factor; an empty string is valid and common.
pub fn to_seed(mnemonic: &Mnemonic, passphrase: &str) -> Zeroizing<[u8; 64]> {
    let seed = mnemonic.to_seed(passphrase);
    Zeroizing::new(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BIP39 test vector from the official spec (first vector, TREZOR passphrase).
    /// mnemonic: "abandon" x11 + "about"
    /// passphrase: "TREZOR"
    /// Expected seed first bytes: c5 52 57 ...
    #[test]
    fn bip39_known_vector() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon \
                      abandon abandon abandon abandon about";
        let mnemonic = parse(phrase).unwrap();
        let seed = to_seed(&mnemonic, "TREZOR");
        assert_eq!(seed.len(), 64);
        // First two bytes of the official test vector
        assert_eq!(seed[0], 0xc5);
        assert_eq!(seed[1], 0x52);
    }

    #[test]
    fn rejects_invalid_mnemonic() {
        assert!(parse("not valid words at all").is_err());
    }

    #[test]
    fn generate_produces_valid_24_word_phrase() {
        let m = generate();
        assert_eq!(m.word_count(), 24);
        // round-trip: parse the phrase back
        let phrase = m.to_string();
        assert!(parse(&phrase).is_ok());
    }

    #[test]
    fn two_generated_mnemonics_differ() {
        let m1 = generate().to_string();
        let m2 = generate().to_string();
        assert_ne!(m1, m2);
    }
}
