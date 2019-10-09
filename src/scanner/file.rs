use std::sync::Arc;
use std::time::Duration;

use futures::prelude::*;
use telegram_bot::types::{GetFile, File};

use crate::{
    config::*,
    state::State,
    util::{self, future::select_true},
};

/// Check whether any of the given files is illegal.
///
/// A list of `GetFile` requests is given, as the actual files should still be downloaded.
pub async fn has_illegal_files(files: Vec<GetFile>, state: State) -> bool {
    // Build a list of file checks, check them concurrently
    select_true(
        files
        .into_iter()
        .map(|file| is_illegal_file(file, state.clone()))
    ).await
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
        .await
    {
        Ok(file) => file,
        Err(err) => {
            println!(
                "failed to request file URL from Telegram, ignoring: {:?}",
                err
            );
            return false;
        }
    };

    // Request the file URL
    let url = file.get_url(state.token()).expect("failed to get file URL");

    // Skip files that are too large
    match file.file_size {
        Some(size) if size > MAX_FILE_SIZE => {
            println!("File to large to audit, ignoring");
            return false;
        },
        _ => {},
    };

    // Do tests based on file extension
    // TODO: better extension test
    let url_lower = url.trim_end().to_lowercase();
    if url_lower.ends_with(".jpg")
        || url_lower.ends_with(".jpeg")
        || url_lower.ends_with(".png")
        || url_lower.ends_with(".gif")
        || url_lower.ends_with(".tiff")
        || url_lower.ends_with(".bmp")
        || url_lower.ends_with(".ico")
        || url_lower.ends_with(".pnm")
        || url_lower.ends_with(".pbm")
        || url_lower.ends_with(".pgm")
        || url_lower.ends_with(".ppm")
        || url_lower.ends_with(".pam")
        || url_lower.ends_with(".webp") {
        if is_illegal_image(file, &url).await {
            return true;
        }
    } else if url_lower.ends_with(".mts")
        || url_lower.ends_with(".avi")
        || url_lower.ends_with(".flv")
        || url_lower.ends_with(".mpeg")
        || url_lower.ends_with(".mp4")
        || url_lower.ends_with(".wmv")
        || url_lower.ends_with(".mov")
        || url_lower.ends_with(".webm") {
        #[cfg(feature = "ffmpeg")]
        {
            if is_illegal_video(file, &url).await {
                return true;
            }
        }
    } else {
        println!("Unhandled file URL: {}", url);
    }

    false
}

/// Check whether the given Telegram image is an illegal file.
async fn is_illegal_image(file: File, url: &str) -> bool {
    // Skip images that are too large
    match file.file_size {
        Some(size) if size > IMAGE_MAX_FILE_SIZE => {
            println!("Image file size too large to audit, ignoring");
            return false;
        },
        _ => {}
    };

    // Download the file to a temporary file to test on
    let path = match util::download::download_temp(&url).await {
        Ok(response) => response.1,
        Err(err) => {
            println!(
                "Failed to download Telegram file to test for illegal content, ignoring: {:?}",
                err
            );
            return false;
        }
    };

    // Test whether the image file is illegal
    super::image::is_illegal_image(Arc::new(path)).await
}

/// Check whether the given Telegram video is an illegal file.
#[cfg(feature = "ffmpeg")]
async fn is_illegal_video(_: File, url: &str) -> bool {
    // Download the file to a temporary file to test on
    let path = match util::download::download_temp(&url).await {
        Ok(response) => response.1,
        Err(err) => {
            println!(
                "Failed to download Telegram file to test for illegal content, ignoring: {:?}",
                err
            );
            return false;
        }
    };

    // Extract video frames
    let frame_file = match util::video::extract_frames(Arc::new(path)).await {
        Ok(frame_file) => frame_file,
        Err(_) => {
            println!("Failed to extract video frames to check, ignoring");
            return false;
        },
    };

    // Test whether the image file is illegal
    super::image::is_illegal_image(frame_file).await
}
