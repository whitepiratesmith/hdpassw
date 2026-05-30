use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::store::site::SiteStore;

/// Resolve the path to the metadata file.
/// XDG: `$XDG_CONFIG_HOME/hdpassw/sites.toml`
/// Fallback: `~/.config/hdpassw/sites.toml`
pub fn default_db_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hdpassw")
        .join("sites.toml")
}

/// Path to the encrypted seed vault.
/// `$XDG_CONFIG_HOME/hdpassw/seed.vault`
pub fn vault_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hdpassw")
        .join("seed.vault")
}

/// Load the site store from disk. Returns an empty store if the file does not exist.
pub fn load(path: &Path) -> Result<SiteStore> {
    if !path.exists() {
        return Ok(SiteStore::default());
    }

    // Warn if world-readable
    #[cfg(unix)]
    check_permissions(path)?;

    let contents = fs::read_to_string(path).map_err(Error::Io)?;
    toml::from_str(&contents).map_err(|e| Error::Store(e.to_string()))
}

/// Write the site store to disk, creating parent directories as needed.
pub fn save(path: &Path, store: &SiteStore) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(Error::Io)?;
    }

    let contents = toml::to_string_pretty(store).map_err(|e| Error::Store(e.to_string()))?;
    fs::write(path, contents).map_err(Error::Io)?;

    // Lock down permissions: owner read/write only
    #[cfg(unix)]
    set_private(path)?;

    Ok(())
}

#[cfg(unix)]
fn check_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(path).map_err(Error::Io)?.permissions().mode();
    if mode & 0o077 != 0 {
        eprintln!(
            "warning: {} is readable by group or others (mode {:o}). \
             Run: chmod 600 {}",
            path.display(),
            mode & 0o777,
            path.display()
        );
    }
    Ok(())
}

#[cfg(unix)]
fn set_private(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms).map_err(Error::Io)?;
    Ok(())
}
