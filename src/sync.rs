use crate::config::Config;
use async_compression::tokio::bufread::GzipDecoder;
use fs2::FileExt;
use futures::TryStreamExt;
use std::{fs::OpenOptions, path::PathBuf};
use tokio_tar::Archive;
use tokio_util::compat::FuturesAsyncReadCompatExt;

pub fn should(config: &Config) -> bool {
    !config.pantry_dir.join("projects").exists()
}

pub async fn replace(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let url = env!("PKGX_PANTRY_TARBALL_URL");
    std::fs::create_dir_all(config.pantry_dir.clone())?;
    download_and_extract_pantry(url, &config.pantry_dir).await
}

async fn download_and_extract_pantry(
    url: &str,
    dest: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = OpenOptions::new()
        .read(true) // Open in read-only mode; no need to write.
        .open(dest)?;

    dir.lock_exclusive()?;

    let rsp = reqwest::get(url).await?.error_for_status()?;

    let stream = rsp.bytes_stream();

    let stream = stream
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read();
    let stream = stream.compat();

    let decoder = GzipDecoder::new(stream);

    // Step 3: Extract the tar archive
    let mut archive = Archive::new(decoder);
    archive.unpack(dest).await?;

    FileExt::unlock(&dir)?;

    Ok(())
}
