use crate::config::Config;
use crate::types::{PackageReq, host};
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::error::Error;
use reqwest::Url;

// Inventory type
pub type Inventory = HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>;

// Custom error for download issues
#[derive(Debug)]
pub struct DownloadError {
    pub status: u16,
    pub src: String,
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Download error: status code {} from {}", self.status, self.src)
    }
}

impl Error for DownloadError {}


// Select function to pick a version
pub async fn select(rq: PackageReq, config: &Config) -> Result<Option<Version>, Box<dyn Error>> {
    let versions = get(&rq, config).await?;

    return Ok(versions.iter().filter(|v| rq.constraint.matches(v)).max().cloned());
}

// Get function to fetch available versions
pub async fn get(rq: &PackageReq, config: &Config) -> Result<Vec<Version>, Box<dyn Error>> {
    let base_url = config.dist_url.clone();

    let (platform, arch) = host();
    let url = Url::parse(&format!(
        "{}/{}/{}/{}/versions.txt",
        base_url,
        rq.project,
        platform,
        arch
    ))?;

    let rsp = reqwest::get(url.clone()).await?;

    if !rsp.status().is_success() {
        return Err(Box::new(DownloadError {
            status: rsp.status().as_u16(),
            src: url.to_string(),
        }));
    }

    let releases = rsp.text().await?;
    let mut versions: Vec<Version> = releases
        .lines()
        .map(|line| line.parse::<Version>())
        .filter_map(Result::ok)
        .collect();

    if versions.is_empty() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("No versions for {}", rq.project),
        )));
    }

    if rq.project == "openssl.org" {
        // Workaround: Remove specific version
        let excluded_version = Version::parse("1.1.118")?;
        versions.retain(|x| x != &excluded_version);
    }

    Ok(versions)
}
