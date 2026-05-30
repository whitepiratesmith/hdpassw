/// Clipboard integration with automatic clearing.
///
/// After copying a password, a background thread waits `timeout_secs`
/// and then clears the clipboard — but only if the content hasn't changed
/// (i.e. the user hasn't copied something else in the meantime).
use std::thread;
use std::time::Duration;

use arboard::Clipboard;

use crate::error::{Error, Result};

/// Copy `text` to the system clipboard, then spawn a thread that clears it
/// after `timeout_secs`. Prints a status line to stderr.
pub fn copy_and_clear(text: &str, timeout_secs: u64) -> Result<()> {
    let mut cb = Clipboard::new().map_err(|e| Error::Clipboard(e.to_string()))?;
    cb.set_text(text).map_err(|e| Error::Clipboard(e.to_string()))?;

    let owned = text.to_owned();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(timeout_secs));
        if let Ok(mut cb) = Clipboard::new() {
            // Only clear if our password is still there
            if cb.get_text().ok().as_deref() == Some(&owned) {
                let _ = cb.set_text("");
            }
        }
    });

    eprintln!("✓  Copied to clipboard — clearing in {timeout_secs}s");
    Ok(())
}
