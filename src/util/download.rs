use std::fs::File;
use std::io::Write;
use std::time::Duration;

use futures::{prelude::*, Future, Stream};
use reqwest::r#async::Client;
use tempfile::{Builder, TempPath};

/// Download a file at the given URL to a temporary file on the system.
/// The downloaded file and path is returned.
///
/// The actual downloaded file is automatically deleted from disk when the last file handle
/// (`File`) is dropped. See `tempfile::NamedTempFile` for more details.
// TODO: make this properly async, the download process isn't at this moment
pub fn download_temp(url: &str) -> impl Future<Item = (File, TempPath), Error = ()> {
    // Build the download client
    // TODO: use a global client instance
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(60))
        .build()
        .expect("failed to build file downloading client");

    // Get file name to suffix temporary downloaded file with
    let name = url.split('/').last().unwrap_or("");;

    // Create temporary file
    let (file, path) = Builder::new()
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

    client
        .get(url)
        .send()
        // TODO: do not drop error here
        .map_err(|err| {
            dbg!(err);
            ()
        })
        .and_then(|response| {
            response
                .into_body()
                // TODO: do not drop error here
                .map_err(|err| {
                    dbg!(err);
                    ()
                })
                .fold(file, |mut download, chunk| {
                    download
                        .write_all(&chunk)
                        .into_future()
                        .map(|_| download)
                        .map_err(|err| {
                            dbg!(err);
                            ()
                        })
                })
        })
        .and_then(|file| {
            // Force sync the file
            // TODO: do not drop error here
            file.sync_all().map(|_| file).map_err(|_| ())
        })
        .map(|file| (file, path))
        .map_err(|_| {
            eprintln!("CATCHED ERR!");
            ()
        })
}
