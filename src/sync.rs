use crate::{config::Config, pantry_db};
use async_compression::tokio::bufread::GzipDecoder;
use fs2::FileExt;
use futures::TryStreamExt;
use std::{error::Error, fs::OpenOptions, path::PathBuf};
use tokio_tar::Archive;
use tokio_util::compat::FuturesAsyncReadCompatExt;

pub fn should(config: &Config) -> bool {
    !config.pantry_dir.join("projects").exists()
}

pub async fn replace(config: &Config) -> Result<rusqlite::Connection, Box<dyn Error>> {
    let url = env!("PKGX_PANTRY_TARBALL_URL");
    let dest = &config.pantry_dir;

    std::fs::create_dir_all(dest.clone())?;
    let dir = OpenOptions::new()
        .read(true) // Open in read-only mode; no need to write.
        .open(dest)?;
    dir.lock_exclusive()?;

    download_and_extract_pantry(url, dest).await?;

    let conn = pantry_db::cache(config)?;

    FileExt::unlock(&dir)?;

    Ok(conn)
}

async fn download_and_extract_pantry(url: &str, dest: &PathBuf) -> Result<(), Box<dyn Error>> {
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

    Ok(())
}
