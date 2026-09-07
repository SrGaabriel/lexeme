use thiserror::Error;

pub mod dict;
pub mod fs;
pub mod lexicon;
pub mod manifest;
pub mod style;
pub mod util;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Failed to download language file: {0}")]
    DownloadError(String),
    #[error("Could not determine home directory, set LEXEME_DIR environment variable to override")]
    HomeDirError,
}

pub type Result<T> = std::result::Result<T, Error>;
