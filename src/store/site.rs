use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::crypto::encode::Charset;

/// Metadata record for one site. Contains no passwords or key material.
/// Safe to store in plaintext, version control, or cloud.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteRecord {
    /// Username or email for this site.
    pub user: String,

    /// Rotation counter. Increment to rotate the password for this site only.
    /// Default: 1. Almost never changes.
    pub counter: u32,

    /// Generated password length.
    pub length: usize,

    /// Character set name.
    pub charset: String,

    /// 4-char hex verifier derived from the site key.
    /// Non-secret. Lets you confirm the correct counter during seed-only recovery.
    pub verifier: String,

    /// When this record was first created.
    pub created: NaiveDate,

    /// When this record was last modified (counter rotated, settings changed, etc.)
    pub modified: NaiveDate,

    /// Optional free-form notes (e.g. "2FA via Authy", "recovery email: …").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl SiteRecord {
    pub fn charset(&self) -> crate::error::Result<Charset> {
        Charset::from_str(&self.charset)
    }
}

/// The root of the TOML metadata file.
#[derive(Debug, Serialize, Deserialize)]
pub struct SiteStore {
    /// Schema version — bump when the format changes to enable migration.
    #[serde(default = "default_version")]
    pub version: u32,

    /// Global rotation generation.  All sites should have counter == rotation.
    /// Increment with `hdpassw rotate`; sites behind are flagged in `ls`.
    #[serde(default = "default_rotation")]
    pub rotation: u32,

    /// Date the current rotation level was set.
    /// Used to warn when 90 days have passed without rotating.
    #[serde(default = "today")]
    pub rotation_since: chrono::NaiveDate,

    #[serde(default)]
    pub sites: std::collections::BTreeMap<String, SiteRecord>,
}

impl Default for SiteStore {
    fn default() -> Self {
        Self {
            version: 1,
            rotation: 1,
            rotation_since: today(),
            sites: Default::default(),
        }
    }
}

fn default_version() -> u32 { 1 }
fn default_rotation() -> u32 { 1 }
fn today() -> chrono::NaiveDate { chrono::Local::now().date_naive() }
