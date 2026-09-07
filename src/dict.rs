use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use zstd::Decoder;

use crate::{Error, Result, fs::languages_dir, lexicon};

pub const REPO_URL: &str = "https://github.com/SrGaabriel/lexeme_dictionaries";

pub fn asset_url(asset_name: &str) -> String {
    format!("{REPO_URL}/releases/latest/download/{asset_name}")
}

pub fn language_url(language: &str) -> String {
    let asset_name = format!("{language}.lex.zst");
    asset_url(&asset_name)
}

pub fn compressed_path(language: &str) -> Result<PathBuf> {
    let dir = languages_dir()?;
    Ok(dir.join(format!("{language}.lex.zst")))
}

pub fn resolve(language: &str) -> Result<reqwest::blocking::Response> {
    let url = language_url(language);
    let response = reqwest::blocking::get(&url)?;
    if !response.status().is_success() {
        return Err(Error::DownloadError(response.status().to_string()));
    }

    Ok(response)
}

pub fn download<T: Read>(language: &str, reader: &mut T) -> Result<()> {
    let dir = languages_dir()?;
    let compressed_path = compressed_path(language)?;
    if compressed_path.exists() {
        return Err(Error::LanguageAlreadyInstalled(language.to_string()));
    }

    let compressed_partial_path = dir.join(format!("{language}.lex.zst.part"));

    let mut compressed_file = File::create(&compressed_partial_path)?;

    std::io::copy(reader, &mut compressed_file)?;
    compressed_file.sync_all()?;
    drop(compressed_file);
    std::fs::rename(&compressed_partial_path, &compressed_path)?;

    Ok(())
}

pub fn remove(language: &str) -> Result<()> {
    let compressed_path = compressed_path(language)?;
    if !compressed_path.exists() {
        return Err(Error::LanguageNotInstalled(language.to_string()));
    }

    std::fs::remove_file(compressed_path)?;
    Ok(())
}

pub fn read(language: &str) -> Result<lexicon::Reader<BufReader<Decoder<'_, BufReader<File>>>>> {
    let compressed_path = compressed_path(language)?;
    if !compressed_path.exists() {
        return Err(Error::LanguageNotInstalled(language.to_string()));
    }

    let decoder = Decoder::new(File::open(&compressed_path)?)?;
    let source = BufReader::new(decoder);
    lexicon::Reader::new(source)
}

pub fn installed_languages() -> Result<Vec<String>> {
    let dir = languages_dir()?;
    let read_dir = std::fs::read_dir(dir)?;
    let mut langs = vec![];
    for read in read_dir {
        let file = read?;
        let Ok(file_name) = file.file_name().into_string() else {
            continue;
        };
        let Some(lang) = file_name.strip_suffix(".lex.zst") else {
            continue;
        };
        langs.push(lang.to_string());
    }
    Ok(langs)
}
