use std::path::PathBuf;

use crate::{Error, Result};

pub fn lexeme_dir() -> Result<PathBuf> {
    let env_var = std::env::var("LEXEME_DIR").ok();
    if let Some(dir) = env_var {
        return Ok(PathBuf::from(dir));
    }

    let home_dir = dirs::home_dir().ok_or_else(|| Error::HomeDirError)?;
    let dir = home_dir.join(".lexeme");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn languages_dir() -> Result<PathBuf> {
    let dir = lexeme_dir()?.join("languages");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
