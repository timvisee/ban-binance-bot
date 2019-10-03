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
pub async fn download_temp(url: &str) -> Result<(File, TempPath), ()> {
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
        // TODO: do not drop error here
        .map_err(|err| {
            dbg!(err);
            ()
        })
        .await?;

    // Write response body chunks to file
    // TODO: do not drop errors here
    while let Some(chunk) = response.chunk().map_err(|err| {
        dbg!(err);
        ()
    }).await? {
        file.write_all(&chunk);
    }

    // Force sync the file
    // TODO: do not drop error here
    file.sync_all();

    Ok((file, path))
}
