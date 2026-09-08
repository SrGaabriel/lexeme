use thiserror::Error;

pub mod dict;
pub mod fs;
pub mod lexicon;
pub mod manifest;
pub mod style;
pub mod util;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("failed to download language file with status {0}")]
    DownloadError(String),
    #[error("could not determine home directory, set LEXEME_DIR environment variable to override")]
    HomeDirError,
    #[error("manifest version mismatch: current version is {0}, downloaded is {1}")]
    LexiconVersionMismatch(u32, u32),
    #[error("language `{0}` not installed")]
    LanguageNotInstalled(String),
    #[error("language `{0}` already installed")]
    LanguageAlreadyInstalled(String),
    #[error("no languages installed. install one with `lexeme lang add`")]
    NoLanguagesInstalled,
    #[error("invalid regex string error")]
    Regex(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
