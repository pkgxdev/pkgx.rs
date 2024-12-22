use flate2::read::GzDecoder;
use std::io::Cursor;
use crate::config::Config;
use std::path::PathBuf;
use tar::Archive;

pub fn replace(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
  let url = "https://dist.pkgx.dev/pantry.tgz";
  std::fs::create_dir_all(config.pantry_dir.clone())?;
  download_and_extract_pantry(url, &config.pantry_dir)
}

fn download_and_extract_pantry(url: &str, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Download the pantry.tgz file
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to download: HTTP {}", response.status()).into());
    }

    // Step 2: Decompress the .gz file
    let decoder = GzDecoder::new(Cursor::new(response.bytes()?));

    // Step 3: Extract the tar archive
    let mut archive = Archive::new(decoder);
    archive.unpack(dest)?;

    Ok(())
}
