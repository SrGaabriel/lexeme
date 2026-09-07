use std::fs::File;

use indicatif::HumanBytes;
use zstd::Decoder;

use crate::{
    Error, Result,
    fs::languages_dir,
    style::{self, Tone},
};

pub const REPO_URL: &str = "https://github.com/SrGaabriel/lexeme_dictionaries";

pub fn asset_url(asset_name: &str) -> String {
    format!("{REPO_URL}/releases/latest/download/{asset_name}")
}

pub fn language_url(language: &str) -> String {
    let asset_name = format!("{language}.lex.zst");
    asset_url(&asset_name)
}

pub fn install(language: String) -> Result<()> {
    let url = language_url(&language);

    let connecting = style::spinner(format!("resolving {language}"));
    let response = reqwest::blocking::get(&url)?;
    connecting.finish_and_clear();

    if !response.status().is_success() {
        return Err(Error::DownloadError(response.status().to_string()));
    }

    let dir = languages_dir()?;

    let compressed_path = dir.join(format!("{language}.lex.zst"));
    let compressed_partial_path = dir.join(format!("{language}.lex.zst.part"));

    let total = response.content_length();
    let download_bar = style::progress_bar(total, format!("downloading {language}"));
    let mut reader = download_bar.wrap_read(response);
    let mut compressed_file = File::create(&compressed_partial_path)?;

    std::io::copy(&mut reader, &mut compressed_file)?;
    compressed_file.sync_all()?;
    drop(compressed_file);

    download_bar.finish_and_clear();
    std::fs::rename(&compressed_partial_path, &compressed_path)?;

    let output_path = dir.join(format!("{language}.lex"));
    let output_partial_path = dir.join(format!("{language}.lex.part"));

    let unzip_bar = style::progress_bar(total, format!("unzipping {language}"));
    let mut decoder = Decoder::new(File::open(&compressed_path)?)?;
    let mut decompressed_file = File::create(&output_partial_path)?;
    let mut writer = unzip_bar.wrap_write(&mut decompressed_file);

    let written = std::io::copy(&mut decoder, &mut writer)?;

    drop(writer);
    decompressed_file.sync_all()?;
    drop(decompressed_file);

    unzip_bar.finish_and_clear();
    std::fs::rename(&output_partial_path, &output_path)?;
    std::fs::remove_file(&compressed_path)?;
    style::status_meta(Tone::Success, "installed", &language, HumanBytes(written));

    Ok(())
}
