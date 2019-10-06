use std::fs::File;
use std::io::Write;
use std::time::Duration;

use futures::prelude::*;
use reqwest::Client;
use tempfile::{Builder, TempPath};

/// Download a file at the given URL to a temporary file on the system.
/// The downloaded file and path is returned.
///
/// The actual downloaded file is automatically deleted from disk when the last file handle
/// (`File`) is dropped. See `tempfile::NamedTempFile` for more details.
// TODO: make this properly async, the download process isn't at this moment
pub async fn download_temp(url: &str) -> Result<(File, TempPath), Error> {
    // Build the download client
    // TODO: use a global client instance
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(60))
        .build()
        .expect("failed to build file downloading client");

    // Get file name to suffix temporary downloaded file with
    let name = url.split('/').last().unwrap_or("");

    // Create temporary file
    let (mut file, path) = Builder::new()
        .suffix(name)
        .tempfile()
        .expect("failed to create file for download")
        .into_parts();

    println!(
        "Downloading '{}' to '{}'...",
        url,
        path.to_str().unwrap_or("?"),
    );

    // TODO: check status code

    // Make the request, obtain the repsonse
    let mut response = client
        .get(url)
        .send()
        .map_err(Error::Request)
        .await?;

    // Write response body chunks to file
    while let Some(chunk) = response.chunk().map_err(Error::Request).await? {
        file.write_all(&chunk).map_err(Error::Write)?;
    }

    // Force sync the file
    let _ = file.sync_all();

    Ok((file, path))
}

/// Represents a download error.
#[derive(Debug)]
pub enum Error {
    /// Download request error.
    ///
    /// An error has occurred while making the download request, or while fetching the download
    /// chunks.
    Request(reqwest::Error),

    /// Download write error.
    ///
    /// Failed to write the file chunks being downloaded to a file on disk.
    Write(std::io::Error),
}
