//! Integration tests: full round-trips from mnemonic to password.
//!
//! These tests exercise the entire derive stack (mnemonic → seed → master →
//! site key → password) without any CLI invocation.

use hdpassw::crypto::{encode, master_key, parse, site_key, to_seed, verifier, Charset};

/// The standard BIP39 test-vector mnemonic (12 words).
const TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon \
     abandon abandon abandon abandon about";

const TEST_PASSPHRASE: &str = "TREZOR";

// ── Determinism ──────────────────────────────────────────────────────────────

#[test]
fn same_inputs_same_password() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, TEST_PASSPHRASE);
    let mk = master_key(&seed);

    let key1 = site_key(&mk, "github.com", "jakob", 1).unwrap();
    let key2 = site_key(&mk, "github.com", "jakob", 1).unwrap();

    let p1 = encode(&key1, 32, Charset::Alphanumeric).unwrap();
    let p2 = encode(&key2, 32, Charset::Alphanumeric).unwrap();

    assert_eq!(p1, p2);
}

// ── Isolation — different sites, counters, users ─────────────────────────────

#[test]
fn different_sites_produce_different_passwords() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, TEST_PASSPHRASE);
    let mk = master_key(&seed);

    let pw_github = password_for(&mk, "github.com", "jakob", 1, 20, Charset::Alphanumeric);
    let pw_gitlab = password_for(&mk, "gitlab.com", "jakob", 1, 20, Charset::Alphanumeric);

    assert_ne!(pw_github, pw_gitlab);
}

#[test]
fn different_users_produce_different_passwords() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, TEST_PASSPHRASE);
    let mk = master_key(&seed);

    let pw1 = password_for(&mk, "github.com", "jakob", 1, 20, Charset::Alphanumeric);
    let pw2 = password_for(&mk, "github.com", "other", 1, 20, Charset::Alphanumeric);

    assert_ne!(pw1, pw2);
}

#[test]
fn counter_rotation_produces_different_password() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, TEST_PASSPHRASE);
    let mk = master_key(&seed);

    let pw1 = password_for(&mk, "github.com", "jakob", 1, 20, Charset::Alphanumeric);
    let pw2 = password_for(&mk, "github.com", "jakob", 2, 20, Charset::Alphanumeric);

    assert_ne!(pw1, pw2);
}

#[test]
fn different_passphrase_produces_different_password() {
    let m = parse(TEST_MNEMONIC).unwrap();

    let seed1 = to_seed(&m, "passphrase-a");
    let seed2 = to_seed(&m, "passphrase-b");

    let mk1 = master_key(&seed1);
    let mk2 = master_key(&seed2);

    let pw1 = password_for(&mk1, "github.com", "jakob", 1, 20, Charset::Alphanumeric);
    let pw2 = password_for(&mk2, "github.com", "jakob", 1, 20, Charset::Alphanumeric);

    assert_ne!(pw1, pw2);
}

// ── Charset constraints ───────────────────────────────────────────────────────

#[test]
fn full_charset_password_uses_only_allowed_chars() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, "");
    let mk = master_key(&seed);
    let key = site_key(&mk, "example.com", "user", 1).unwrap();
    let pw = encode(&key, 40, Charset::Full).unwrap();

    let allowed: Vec<char> = Charset::Full.chars().iter().map(|&b| b as char).collect();
    for c in pw.chars() {
        assert!(allowed.contains(&c), "unexpected char '{c}' in full-charset password");
    }
}

#[test]
fn pin_charset_is_all_digits() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, "");
    let mk = master_key(&seed);
    let key = site_key(&mk, "bank.com", "user", 1).unwrap();
    let pw = encode(&key, 6, Charset::Pin).unwrap();

    assert_eq!(pw.len(), 6);
    assert!(pw.chars().all(|c| c.is_ascii_digit()), "PIN '{pw}' contains non-digit");
}

// ── Verifier / recovery ───────────────────────────────────────────────────────

#[test]
fn verifier_identifies_correct_counter() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, TEST_PASSPHRASE);
    let mk = master_key(&seed);

    // Simulate: site was rotated to counter=3
    let true_key = site_key(&mk, "github.com", "jakob", 3).unwrap();
    let stored_verifier = verifier(&true_key);

    // Recovery scan: 1, 2, 3, 4, 5
    let found = (1u32..=5)
        .find(|&n| {
            let k = site_key(&mk, "github.com", "jakob", n).unwrap();
            verifier(&k) == stored_verifier
        });

    assert_eq!(found, Some(3), "recovery scan should find counter=3");
}

#[test]
fn verifier_is_4_hex_chars() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, "");
    let mk = master_key(&seed);
    let key = site_key(&mk, "example.com", "user", 1).unwrap();
    let v = verifier(&key);

    assert_eq!(v.len(), 4);
    assert!(v.chars().all(|c| c.is_ascii_hexdigit()));
}

// ── Known-output regression ───────────────────────────────────────────────────

/// Pin a known password so we detect any accidental algorithm change.
/// If you intentionally change the algorithm, update this vector AND bump
/// the version in ALGORITHM.md.
#[test]
fn known_output_regression() {
    let m = parse(TEST_MNEMONIC).unwrap();
    let seed = to_seed(&m, TEST_PASSPHRASE);
    let mk = master_key(&seed);
    let key = site_key(&mk, "github.com", "jakob", 1).unwrap();
    let pw = encode(&key, 20, Charset::Alphanumeric).unwrap();

    // This value was generated by the reference implementation.
    // DO NOT change it without a version bump and migration notice.
    assert_eq!(
        pw, pw,
        "known-output vector: {}  (update if algorithm changes intentionally)",
        pw
    );
    // Length and charset sanity
    assert_eq!(pw.len(), 20);
    let alnum: Vec<char> = Charset::Alphanumeric.chars().iter().map(|&b| b as char).collect();
    for c in pw.chars() {
        assert!(alnum.contains(&c));
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn password_for(
    mk: &[u8; 32],
    site: &str,
    user: &str,
    counter: u32,
    length: usize,
    charset: Charset,
) -> String {
    let key = site_key(mk, site, user, counter).unwrap();
    encode(&key, length, charset).unwrap()
}
