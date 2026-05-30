/// Key derivation layer.
///
/// Algorithm:
///   1. seed[0..32]  → scrypt(N=2^17, r=8, p=1, salt=APP_SALT) → 32-byte master key
///   2. master key   → BLAKE3-KDF(context, master ∥ info)       → 32-byte site key
///
/// scrypt provides memory-hard resistance against brute-forcing a weak passphrase.
/// BLAKE3's native key-derivation mode handles all per-site derivation with
/// built-in domain separation via a hardcoded context string.
use scrypt::{scrypt, Params};
use zeroize::Zeroizing;

use crate::error::Result;

/// Fixed application salt for the scrypt step.
/// Domain-separates hdpassw from any other tool that might process the same
/// BIP39 seed bytes (e.g. a wallet).
const APP_SALT: &[u8] = b"hdpassw-v1";

/// BLAKE3 context for the site-key derivation step.
/// Must be globally unique, hardcoded, and never change for a given version.
const CTX_SITE_KEY: &str = "hdpassw 2026-05-24 site-key";

/// Derive the 32-byte master key from the 64-byte BIP39 seed.
///
/// scrypt parameters: N=2^17 (128 MiB), r=8, p=1.
/// This is the OWASP-recommended interactive minimum. The cost is paid once
/// per session unlock; typical wall-clock time ≈ 0.5 s on modern hardware.
///
/// Only `seed[0..32]` is used as the scrypt password; the upper 32 bytes are
/// reserved for future key types.
pub fn master_key(seed: &[u8; 64]) -> Zeroizing<[u8; 32]> {
    let params = Params::new(17, 8, 1, 32).expect("valid scrypt params");
    derive_master_with_params(seed, &params)
}

/// Inner implementation that accepts explicit scrypt params.
/// Exposed as `pub(crate)` so unit tests can pass cheap params without a
/// feature flag, while the public `master_key` always uses the secure defaults.
pub(crate) fn derive_master_with_params(seed: &[u8; 64], params: &Params) -> Zeroizing<[u8; 32]> {
    let mut out = Zeroizing::new([0u8; 32]);
    scrypt(&seed[..32], APP_SALT, params, out.as_mut())
        .expect("scrypt output length is always valid");
    out
}

/// Derive a 32-byte site key from the master key using BLAKE3 in KDF mode.
///
/// The key material fed into BLAKE3 is `master ∥ info` where info encodes
/// the site name, username, and rotation counter. BLAKE3's context string
/// provides domain separation between this step and any other BLAKE3 usage.
///
/// `site`    — domain or application name, e.g. `"github.com"`
/// `user`    — username or email for this site
/// `counter` — rotation counter; increment to rotate without changing master key
pub fn site_key(
    master: &[u8; 32],
    site: &str,
    user: &str,
    counter: u32,
) -> Result<Zeroizing<[u8; 32]>> {
    let info = format!("site:{site}|user:{user}|n:{counter}");
    let mut ikm = Zeroizing::new(Vec::<u8>::with_capacity(32 + info.len()));
    ikm.extend_from_slice(master);
    ikm.extend_from_slice(info.as_bytes());
    Ok(Zeroizing::new(blake3::derive_key(CTX_SITE_KEY, ikm.as_slice())))
}

/// Derive a 4-character verifier from a site key.
///
/// Stored openly in the metadata file. Lets `hdpassw recover` confirm the
/// correct counter by scanning `n = 1, 2, 3, …` without revealing any
/// password material.
///
/// Format: 4 lowercase hex chars, e.g. `"a3f9"`.
pub fn verifier(site_key: &[u8; 32]) -> String {
    format!("{:02x}{:02x}", site_key[0], site_key[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal scrypt params for fast unit tests (N=2^4, not 2^17).
    fn fast_params() -> Params {
        Params::new(4, 8, 1, 32).unwrap()
    }

    fn dummy_seed() -> [u8; 64] {
        let mut s = [0u8; 64];
        s[0] = 0xde;
        s[1] = 0xad;
        s
    }

    #[test]
    fn master_key_is_deterministic() {
        let seed = dummy_seed();
        let p = fast_params();
        let k1 = derive_master_with_params(&seed, &p);
        let k2 = derive_master_with_params(&seed, &p);
        assert_eq!(k1.as_ref(), k2.as_ref());
    }

    #[test]
    fn site_key_is_deterministic() {
        let seed = dummy_seed();
        let mk = derive_master_with_params(&seed, &fast_params());
        let k1 = site_key(&mk, "github.com", "jakob", 1).unwrap();
        let k2 = site_key(&mk, "github.com", "jakob", 1).unwrap();
        assert_eq!(k1.as_ref(), k2.as_ref());
    }

    #[test]
    fn different_sites_produce_different_keys() {
        let seed = dummy_seed();
        let mk = derive_master_with_params(&seed, &fast_params());
        let k_gh = site_key(&mk, "github.com", "jakob", 1).unwrap();
        let k_gl = site_key(&mk, "gitlab.com", "jakob", 1).unwrap();
        assert_ne!(k_gh.as_ref(), k_gl.as_ref());
    }

    #[test]
    fn counter_rotation_changes_key() {
        let seed = dummy_seed();
        let mk = derive_master_with_params(&seed, &fast_params());
        let k1 = site_key(&mk, "github.com", "jakob", 1).unwrap();
        let k2 = site_key(&mk, "github.com", "jakob", 2).unwrap();
        assert_ne!(k1.as_ref(), k2.as_ref());
    }

    #[test]
    fn verifier_is_4_hex_chars() {
        let seed = dummy_seed();
        let mk = derive_master_with_params(&seed, &fast_params());
        let sk = site_key(&mk, "example.com", "user", 1).unwrap();
        let v = verifier(&sk);
        assert_eq!(v.len(), 4);
        assert!(v.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
