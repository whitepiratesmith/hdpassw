use std::fs;
use std::io::{self, BufRead, Write};

use hdpassw::crypto;
use hdpassw::error::{Error, Result};
use hdpassw::store::vault_path;

use crate::cli::SeedCommand;

pub fn run(cmd: SeedCommand) -> Result<()> {
    match cmd {
        SeedCommand::New => new(),
        SeedCommand::Check => check(),
        SeedCommand::Restore => restore(),
        SeedCommand::Remove => remove(),
    }
}

/// First-run entry point: ask whether to create a new seed phrase or
/// restore an existing one, then hand off to the matching `seed` flow.
pub fn init() -> Result<()> {
    eprintln!();
    eprintln!("  hdpassw setup");
    eprintln!("  ─────────────");
    eprintln!("  1) Create a new seed phrase");
    eprintln!("  2) Restore an existing seed phrase");
    eprintln!();

    loop {
        eprint!("  Choice [1/2]: ");
        io::stderr().flush().map_err(Error::Io)?;

        let mut line = String::new();
        io::stdin().lock().read_line(&mut line).map_err(Error::Io)?;

        match line.trim() {
            "1" => return new(),
            "2" => return restore(),
            _ => eprintln!("  Please enter 1 or 2."),
        }
    }
}

fn new() -> Result<()> {
    let mnemonic = crypto::generate();
    let words: Vec<&str> = mnemonic.words().collect();
    let phrase = mnemonic.to_string();

    eprintln!();
    eprintln!("  YOUR SEED PHRASE  ({} words)", words.len());
    eprintln!();

    for (i, chunk) in words.chunks(4).enumerate() {
        let line = chunk
            .iter()
            .enumerate()
            .map(|(j, w)| format!("{:2}. {:<10}", i * 4 + j + 1, w))
            .collect::<Vec<_>>()
            .join("  ");
        eprintln!("  {line}");
    }

    eprintln!();
    eprintln!("  Write this down and store it offline.");
    eprintln!("  Never share it or type it anywhere except hdpassw.");
    eprintln!();

    offer_vault_save(&phrase)
}

fn check() -> Result<()> {
    let phrase = rpassword::prompt_password("Enter mnemonic: ")
        .map_err(Error::Io)?;

    let phrase = phrase.trim().to_string();
    match crypto::parse(&phrase) {
        Ok(m) => {
            eprintln!("✓  Valid BIP39 mnemonic ({} words).", m.word_count());
        }
        Err(e) => {
            eprintln!("✗  Invalid: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn restore() -> Result<()> {
    eprintln!();
    eprintln!("  Seed phrase restore");
    eprintln!("  ───────────────────");
    eprintln!("  Enter all words separated by spaces, then press Enter.");
    eprintln!();

    let phrase = loop {
        eprint!("  Seed phrase: ");
        io::stderr().flush().map_err(Error::Io)?;

        let mut line = String::new();
        io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(Error::Io)?;

        let words: Vec<&str> = line.split_whitespace().collect();

        if words.is_empty() {
            eprintln!("  (nothing entered, try again)");
            continue;
        }

        let mut bad: Vec<String> = Vec::new();
        for w in &words {
            if bip39::Language::English.find_word(&w.to_lowercase()).is_none() {
                bad.push(w.to_string());
            }
        }
        if !bad.is_empty() {
            eprintln!();
            eprintln!("  ✗  Unknown word(s): {}", bad.join(", "));
            eprintln!("     Check spelling and try again.");
            eprintln!();
            continue;
        }

        let phrase = words
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        eprintln!();
        match crypto::parse(&phrase) {
            Ok(m) => {
                eprintln!("  ✓  Valid {}-word BIP39 seed phrase.", m.word_count());
                break phrase;
            }
            Err(e) => {
                eprintln!("  ✗  {e}");
                eprintln!();
                eprintln!(
                    "  All words were recognised but the phrase is invalid ({} words entered).",
                    words.len()
                );
                eprintln!("  Check the word count and order, then try again.");
                eprintln!();
            }
        }
    };

    eprintln!();
    offer_vault_save(&phrase)
}

fn remove() -> Result<()> {
    let path = vault_path();
    if !path.exists() {
        eprintln!("  No vault found at {}.", path.display());
        return Ok(());
    }

    eprint!("  Remove encrypted vault at {}? [y/N]: ", path.display());
    io::stderr().flush().map_err(Error::Io)?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).map_err(Error::Io)?;

    if line.trim().eq_ignore_ascii_case("y") {
        fs::remove_file(&path).map_err(Error::Io)?;
        eprintln!("  ✓  Vault removed.");
        eprintln!("     hdpassw will now prompt for your full seed phrase on each use.");
    } else {
        eprintln!("  Cancelled.");
    }

    Ok(())
}

/// After showing or accepting a seed phrase, offer to save it to the vault.
fn offer_vault_save(phrase: &str) -> Result<()> {
    let path = vault_path();

    if path.exists() {
        eprintln!("  A vault already exists at {}.", path.display());
        eprint!("  Overwrite it with this seed? [y/N]: ");
    } else {
        eprint!("  Save seed to encrypted vault? [Y/n]: ");
    }
    io::stderr().flush().map_err(Error::Io)?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).map_err(Error::Io)?;
    let answer = line.trim().to_lowercase();

    // Default is Y for a new vault, N for overwrite
    let save = if path.exists() {
        answer == "y"
    } else {
        answer != "n"
    };

    if !save {
        eprintln!();
        eprintln!("  Vault not saved. You will be asked for your seed phrase on every use.");
        eprintln!();
        return Ok(());
    }

    // Prompt for vault password (twice to confirm)
    let password = loop {
        let pw1 = rpassword::prompt_password("  Vault password: ").map_err(Error::Io)?;
        if pw1.is_empty() {
            eprintln!("  Password cannot be empty, try again.");
            continue;
        }
        let pw2 = rpassword::prompt_password("  Confirm password: ").map_err(Error::Io)?;
        if pw1 != pw2 {
            eprintln!("  Passwords do not match, try again.");
            continue;
        }
        break pw1;
    };

    eprintln!("  Deriving key… (this takes a moment)");
    let vault_data = crypto::vault_seal(phrase, &password)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(Error::Io)?;
    }
    fs::write(&path, vault_data).map_err(Error::Io)?;

    // Lock down: owner read/write only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .map_err(Error::Io)?;
    }

    eprintln!();
    eprintln!("  ✓  Vault saved to {}.", path.display());
    eprintln!("     From now on hdpassw will ask for your vault password instead");
    eprintln!("     of the full seed phrase.");
    eprintln!();

    Ok(())
}
