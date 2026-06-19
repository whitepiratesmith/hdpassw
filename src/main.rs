mod cli;
mod commands;

use std::fs;

use clap::Parser;

use cli::{Cli, Command};
use hdpassw::error::{self, Result};
use hdpassw::session::Session;
use hdpassw::store::{default_db_path, vault_path};
use hdpassw::crypto;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Warn loudly if running as root
    #[cfg(unix)]
    if unsafe { libc::getuid() } == 0 {
        eprintln!("warning: running as root is not recommended");
    }

    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);

    match cli.command {
        // Commands that don't need the master key
        Command::Init => commands::seed::init(),
        Command::Seed(seed_cmd) => commands::seed::run(seed_cmd),
        Command::Ls(args) => commands::ls::run(args, &db_path),
        Command::Rm(args) => commands::rm::run(args, &db_path),
        Command::Export => commands::export::run(&db_path),
        Command::Rotate => commands::rotate::run(&db_path),

        // Commands that require unlocking the session
        cmd => {
            let phrase = load_phrase(cli.passphrase.as_deref())?;
            let session = Session::unlock(&phrase, cli.passphrase)?;

            match cmd {
                Command::Gen(args) => commands::r#gen::run(args, &db_path, &session),
                Command::Add(args) => commands::add::run(args, &db_path, &session),
                Command::Recover(args) => commands::recover::run(args, &session),
                // Already handled above
                Command::Init
                | Command::Seed(_)
                | Command::Ls(_)
                | Command::Rm(_)
                | Command::Export
                | Command::Rotate => {
                    unreachable!()
                }
            }
        }
    }
}

/// Load the seed phrase: vault → env var → manual prompt (fallback).
fn load_phrase(_passphrase_hint: Option<&str>) -> Result<String> {
    // 1. Encrypted vault on disk
    let vpath = vault_path();
    if vpath.exists() {
        let data = fs::read(&vpath).map_err(error::Error::Io)?;
        let password = rpassword::prompt_password("Vault password: ")
            .map_err(error::Error::Io)?;
        let phrase = crypto::vault_open(&data, &password)?;
        return Ok(phrase.to_string());
    }

    // 2. Environment variable (scripting/testing)
    if let Ok(m) = std::env::var("HDPASSW_MNEMONIC") {
        eprintln!("warning: using mnemonic from HDPASSW_MNEMONIC (clear it after use)");
        return Ok(m);
    }

    // 3. Manual entry
    rpassword::prompt_password("Seed phrase: ").map_err(error::Error::Io)
}
