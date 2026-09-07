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
    #[error("Manifest version mismatch: current version is {0}, downloaded is {1}")]
    LexiconVersionMismatch(u32, u32),
    #[error("Language not installed: {0}")]
    LanguageNotInstalled(String),
    #[error("Unknown language: {0}")]
    UnknownLanguage(String),
    #[error("Invalid Regex string error: {0}")]
    Regex(#[from] regex::Error)
}

pub type Result<T> = std::result::Result<T, Error>;
