use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("invalid charset: {0}")]
    InvalidCharset(String),

    #[error("site not found: {0}")]
    SiteNotFound(String),

    #[error("store error: {0}")]
    Store(String),

    #[error("vault error: {0}")]
    Vault(String),

    #[error("clipboard error: {0}")]
    Clipboard(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
