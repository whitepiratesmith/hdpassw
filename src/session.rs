/// Session management: prompt for passphrase, derive and hold master key.
///
/// The master key is stored in a `Zeroizing` buffer. It is never written to
/// disk and is zeroed on drop. The passphrase is read via `rpassword` so it
/// never appears in the terminal scroll-back or process argument list.
use zeroize::Zeroizing;

use crate::crypto;
use crate::error::Result;

/// A live session holding the derived master key.
/// Drop to zero the key.
pub struct Session {
    master: Zeroizing<[u8; 32]>,
}

impl Session {
    /// Prompt the user for their passphrase and derive the master key.
    /// `phrase` — the BIP39 mnemonic (already validated)
    /// `passphrase_env` — if `Some`, use this value instead of prompting
    ///                    (for scripting via `$HDPASSW_PASSPHRASE`)
    pub fn unlock(phrase: &str, passphrase_env: Option<String>) -> Result<Self> {
        let mnemonic = crypto::parse(phrase)?;

        let passphrase = passphrase_env.unwrap_or_default();

        let seed = crypto::to_seed(&mnemonic, &passphrase);
        let master = crypto::master_key(&seed);

        // seed is dropped here — Zeroizing clears it
        Ok(Self { master })
    }

    /// Derive the site key for the given site/user/counter triple.
    pub fn site_key(
        &self,
        site: &str,
        user: &str,
        counter: u32,
    ) -> Result<Zeroizing<[u8; 32]>> {
        crypto::site_key(&self.master, site, user, counter)
    }

    /// Derive verifier string for the given site key.
    pub fn verifier(&self, site: &str, user: &str, counter: u32) -> Result<String> {
        let key = self.site_key(site, user, counter)?;
        Ok(crypto::verifier(&key))
    }
}
