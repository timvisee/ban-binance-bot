use std::sync::Arc;
use std::time::Duration;

use futures::prelude::*;
use telegram_bot::GetFile;

use crate::{config::*, state::State, util};

/// Check whether any of the given files is illegal.
///
/// A list of `GetFile` requests is given, as the actual files should still be downloaded.
pub async fn has_illegal_files(
    mut files: Vec<GetFile>,
    state: State,
) -> bool {
    // TODO: reverse list of files here (pick biggest image first)?
    files.reverse();

    // Test all files in order, return if any is illegal
    // TODO: use iterator
    for file in files {
        if is_illegal_file(file, state.clone()).await {
            return true
        }
    }

    false
}

/// Check whether the given file is illegal.
///
/// A `GetFile` request is given, as the actual file should still be downloaded.
pub async fn is_illegal_file(file: GetFile, state: State) -> bool {
    // Request the file from Telegram
    let file = match state
            .telegram_client()
            .send_timeout(file, Duration::from_secs(30))
            // TODO: do not drop error here
            .map_err(|err| {
                dbg!(err);
                ()
            })
            .await {
        Ok(file) => file,
        Err(err) => {
            println!("failed to request file URL from Telegram, ignoring: {:?}", err);
            return false;
        },
    };

    // Request the file URL
    // TODO: do not error here
    let url = file.get_url(state.token()).expect("failed to get file URL");

    // Skip files that are too large
    match file.file_size {
        Some(size) if size > MAX_FILE_SIZE => return false,
        _ => {}
    };

    // Download image files based on extension to test for legality
    // TODO: better extension test
    if url.ends_with(".jpg")
        || url.ends_with(".jpeg")
        || url.ends_with(".png")
        || url.ends_with(".webp")
    {
        // Download the file to a temporary file to test on
        let (_file, path) = match util::download::download_temp(&url).await {
            Ok(response) => response,
            Err(err) => {
                println!("failed to download Telegram file to test for illegal content, ignoring: {:?}", err);
                return false;
            },
        };

        // Test whether the image file is illegal
        if super::image::is_illegal_image(Arc::new(path)).await {
            return true;
        }
    }

    false
}
