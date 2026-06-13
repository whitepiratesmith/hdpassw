//! Shared password-manager operations used by both the CLI commands and the GUI.
//!
//! Keeping this logic in one place ensures the CLI and GUI can never derive
//! different passwords or write different-looking site records for the same
//! inputs.
use chrono::Local;

use crate::crypto::encode::{self, Charset};
use crate::error::Result;
use crate::session::Session;
use crate::store::{SiteRecord, SiteStore};

/// Derive the password for `site`/`user`/`counter` using `charset`/`length`.
pub fn generate_password(
    session: &Session,
    site: &str,
    user: &str,
    counter: u32,
    length: usize,
    charset: Charset,
) -> Result<String> {
    let key = session.site_key(site, user, counter)?;
    encode::encode(&key, length, charset)
}

/// Insert or update a site record, computing its verifier from `session`.
/// Preserves the original `created` date if the site already exists.
/// Does not persist `store` to disk — call `store::save` afterwards.
pub fn upsert_site(
    store: &mut SiteStore,
    session: &Session,
    site: &str,
    user: &str,
    counter: u32,
    length: usize,
    charset: Charset,
    notes: Option<String>,
) -> Result<()> {
    let today = Local::now().date_naive();
    let verifier = session.verifier(site, user, counter)?;
    let created = store.sites.get(site).map(|r| r.created).unwrap_or(today);

    let record = SiteRecord {
        user: user.to_string(),
        counter,
        length,
        charset: charset.as_str().to_string(),
        verifier,
        created,
        modified: today,
        notes,
    };

    store.sites.insert(site.to_string(), record);
    Ok(())
}
