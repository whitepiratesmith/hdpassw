/// Encrypted seed vault.
///
/// The seed phrase is sealed with ChaCha20-Poly1305, key derived from
/// the user's vault password via Argon2id.
///
/// File format (TOML, stored at ~/.config/hdpassw/seed.vault):
///
///   version    = 1
///   m_cost     = 65536   # Argon2id memory (KiB)
///   t_cost     = 3       # Argon2id iterations
///   p_cost     = 1       # Argon2id parallelism
///   salt       = "<hex>" # 32-byte random salt
///   nonce      = "<hex>" # 12-byte random nonce
///   ciphertext = "<hex>" # encrypted mnemonic + 16-byte Poly1305 tag

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::error::{Error, Result};

const M_COST: u32 = 65536; // 64 MiB
const T_COST: u32 = 3;
const P_COST: u32 = 1;

#[derive(Serialize, Deserialize)]
struct VaultFile {
    version: u32,
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
    salt: String,
    nonce: String,
    ciphertext: String,
}

/// Encrypt `phrase` with `password` and return the TOML vault file bytes.
pub fn seal(phrase: &str, password: &str) -> Result<Vec<u8>> {
    let mut salt = [0u8; 32];
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt, M_COST, T_COST, P_COST)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&*key));
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), phrase.as_bytes())
        .map_err(|_| Error::Vault("encryption failed".into()))?;

    let vf = VaultFile {
        version: 1,
        m_cost: M_COST,
        t_cost: T_COST,
        p_cost: P_COST,
        salt: hex::encode(salt),
        nonce: hex::encode(nonce_bytes),
        ciphertext: hex::encode(ciphertext),
    };

    toml::to_string_pretty(&vf)
        .map(|s| s.into_bytes())
        .map_err(|e| Error::Vault(e.to_string()))
}

/// Decrypt the vault file bytes with `password` and return the seed phrase.
/// Returns `Err` with a clear message on wrong password or corrupt file.
pub fn open(data: &[u8], password: &str) -> Result<Zeroizing<String>> {
    let vf: VaultFile = toml::from_str(
        std::str::from_utf8(data).map_err(|_| Error::Vault("vault file is not valid UTF-8".into()))?,
    )
    .map_err(|e| Error::Vault(format!("vault file is corrupt: {e}")))?;

    let salt = hex::decode(&vf.salt)
        .map_err(|_| Error::Vault("vault file is corrupt (bad salt)".into()))?;
    let nonce_bytes = hex::decode(&vf.nonce)
        .map_err(|_| Error::Vault("vault file is corrupt (bad nonce)".into()))?;
    let ciphertext = hex::decode(&vf.ciphertext)
        .map_err(|_| Error::Vault("vault file is corrupt (bad ciphertext)".into()))?;

    let key = derive_key(password, &salt, vf.m_cost, vf.t_cost, vf.p_cost)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&*key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_slice())
        .map_err(|_| Error::Vault("incorrect password".into()))?;

    String::from_utf8(plaintext)
        .map(Zeroizing::new)
        .map_err(|_| Error::Vault("decrypted content is not valid UTF-8".into()))
}

fn derive_key(
    password: &str,
    salt: &[u8],
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
) -> Result<Zeroizing<[u8; 32]>> {
    let params = Params::new(m_cost, t_cost, p_cost, Some(32))
        .map_err(|e| Error::Vault(format!("invalid Argon2 params: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|e| Error::Vault(format!("key derivation failed: {e}")))?;
    Ok(key)
}
