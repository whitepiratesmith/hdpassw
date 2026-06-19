use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// hdpassw — deterministic password manager backed by a BIP39 seed phrase.
///
/// Passwords are never stored. Every password is derived on the fly from
/// your seed phrase + site name. Back up one seed phrase; recover everything.
#[derive(Parser)]
#[command(name = "hdpassw", version, about, long_about = None)]
pub struct Cli {
    /// Path to the metadata file (default: ~/.config/hdpassw/sites.toml)
    #[arg(long, global = true, value_name = "FILE")]
    pub db: Option<PathBuf>,

    /// Read passphrase from environment instead of prompting.
    /// Equivalent to setting $HDPASSW_PASSPHRASE.
    #[arg(long, global = true, env = "HDPASSW_PASSPHRASE", hide_env_values = true)]
    pub passphrase: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Get started: create a new seed phrase or restore an existing one.
    ///
    /// Equivalent to `seed new` / `seed restore`, but asks which you want
    /// instead of you having to know the subcommand up front.
    Init,

    /// Generate (and copy) the password for a site.
    Gen(GenArgs),

    /// Add or update a site record in the metadata file.
    Add(AddArgs),

    /// List all known sites.
    Ls(LsArgs),

    /// Remove a site record from the metadata file.
    Rm(RmArgs),

    /// Seed phrase management.
    #[command(subcommand)]
    Seed(SeedCommand),

    /// Export metadata to stdout (no passwords).
    Export,

    /// Bump the global rotation counter and list sites that need regenerating.
    ///
    /// After rotating, run `hdpassw gen <site>` for each listed site to get
    /// the new password and update your account.
    Rotate,

    /// Recover counter for a site by scanning from a seed alone.
    Recover(RecoverArgs),
}

// ── gen ──────────────────────────────────────────────────────────────────────

/// Generate the password for a site.
///
/// If the site exists in the metadata file its settings are used automatically.
/// Override any setting on the command line.
#[derive(Parser)]
pub struct GenArgs {
    /// Site name (domain or application, e.g. "github.com")
    pub site: String,

    /// Username or email for this site
    #[arg(short, long)]
    pub user: Option<String>,

    /// Rotation counter (default: from metadata, or 1)
    #[arg(short = 'n', long)]
    pub counter: Option<u32>,

    /// Password length (default: from metadata, or 32)
    #[arg(short, long)]
    pub length: Option<usize>,

    /// Character set: alpha | alphanumeric | full | pin | hex
    #[arg(long)]
    pub charset: Option<String>,

    /// Print to stdout instead of copying to clipboard
    #[arg(long)]
    pub reveal: bool,

    /// Seconds before clipboard is cleared (default: 30)
    #[arg(long, default_value = "30")]
    pub clip_timeout: u64,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

// ── add ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
pub struct AddArgs {
    /// Site name
    pub site: String,

    /// Username or email
    #[arg(short, long)]
    pub user: String,

    /// Rotation counter (default: 1)
    #[arg(short = 'n', long, default_value = "1")]
    pub counter: u32,

    /// Password length (default: 32)
    #[arg(short, long, default_value = "32")]
    pub length: usize,

    /// Character set (default: alphanumeric)
    #[arg(long, default_value = "alphanumeric")]
    pub charset: String,

    /// Optional notes
    #[arg(long)]
    pub notes: Option<String>,
}

// ── ls ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
pub struct LsArgs {
    /// Filter by substring in site name
    pub filter: Option<String>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

// ── rm ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
pub struct RmArgs {
    /// Site name to remove
    pub site: String,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub yes: bool,
}

// ── seed ─────────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum SeedCommand {
    /// Generate a new 24-word BIP39 mnemonic and print it.
    New,

    /// Validate a mnemonic without deriving any keys.
    Check,

    /// Restore from an existing seed phrase — enter words separated by spaces.
    ///
    /// Validates every word against the BIP39 list and checks the full
    /// phrase checksum. Offers to save the phrase to the encrypted vault.
    Restore,

    /// Remove the encrypted seed vault from disk.
    ///
    /// After removal, hdpassw will prompt for the full seed phrase again
    /// on every command that needs it.
    Remove,
}

// ── recover ──────────────────────────────────────────────────────────────────

#[derive(Parser)]
pub struct RecoverArgs {
    /// Site name to scan
    pub site: String,

    /// Username to use
    #[arg(short, long)]
    pub user: String,

    /// 4-char hex verifier to match (from your metadata backup)
    #[arg(long)]
    pub verifier: String,

    /// Maximum counter value to try (default: 20)
    #[arg(long, default_value = "20")]
    pub max_counter: u32,
}
