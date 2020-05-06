use std::sync::Arc;
use std::time::Duration;

use futures::prelude::*;
use telegram_bot::types::{GetFile, File};
use url::Url;

use crate::{
    config::Scanner,
    // TODO: remove this!
    config::*,
    state::State,
    util::{self, future::select_true},
};

/// Check whether any of the given files is illegal.
///
/// A list of `GetFile` requests is given, as the actual files should still be downloaded.
pub async fn has_illegal_files(config: &Scanner, files: Vec<GetFile>, state: State) -> bool {
    // Build a list of file checks, check them concurrently
    select_true(
        files
        .into_iter()
        .map(|file| is_illegal_file(config, file, state.clone()))
    ).await
}

/// Check whether the given file is illegal.
///
/// A `GetFile` request is given, as the actual file should still be downloaded.
pub async fn is_illegal_file(config: &Scanner, file: GetFile, state: State) -> bool {
    // Request download URL for Telegram file
    let (file, url) = match request_telegram_file_url(file, state).await {
        Ok(data) => data,
        Err(_) => {
            warn!("Failed to get Telegram API file URL, could not audit, assuming safe");
            return false;
        },
    };

    // Skip files that are too large
    match file.file_size {
        Some(size) if size > MAX_FILE_SIZE => {
            info!("File to large to audit, assuming safe");
            return false;
        },
        _ => {},
    };

    // Do tests based on file extension
    // TODO: better extension test
    let url_path = url
        .path_segments()
        .and_then(|s| s.last())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "".into());
    if url_path.ends_with(".jpg")
        || url_path.ends_with(".jpeg")
        || url_path.ends_with(".png")
        || url_path.ends_with(".gif")
        || url_path.ends_with(".tiff")
        || url_path.ends_with(".bmp")
        || url_path.ends_with(".ico")
        || url_path.ends_with(".pnm")
        || url_path.ends_with(".pbm")
        || url_path.ends_with(".pgm")
        || url_path.ends_with(".ppm")
        || url_path.ends_with(".pam")
        || url_path.ends_with(".webp") {
        if is_illegal_image(config, file, &url).await {
            return true;
        }
    } else if url_path.ends_with(".mts")
        || url_path.ends_with(".avi")
        || url_path.ends_with(".flv")
        || url_path.ends_with(".mpeg")
        || url_path.ends_with(".mp4")
        || url_path.ends_with(".wmv")
        || url_path.ends_with(".mov")
        || url_path.ends_with(".webm") {
        #[cfg(feature = "ffmpeg")]
        {
            if is_illegal_video(config, file, &url).await {
                return true;
            }
        }
    } else if url_path.ends_with(".tgs") {
        debug!("No scanner for animated Telegram stickers, assuming safe: {}", url);
    } else {
        warn!("No scanners to audit file type, assuming safe: {}", url);
    }

    false
}

/// Get `File` for Telegram API `GetFile`.
async fn request_telegram_file(file: GetFile, state: State) -> Result<File, ()> {
    state
        .telegram_client()
        .send_timeout(file, Duration::from_secs(30))
        .map_err(|err| {
            error!("Failed to send file data request to Telegram API: {:?}", err);
            ()
        })
        .await
        .map_err(|err| {
            error!("Failed to request file data from Telegram API: {:?}", err);
            ()
        })
        .and_then(|file| file.ok_or_else(|| {
            error!("Expected file data from Telegram API, but did not receive anything");
            ()
        }))
}

/// Get download URL for Telegram API `GetFile`.
async fn request_telegram_file_url(file: GetFile, state: State) -> Result<(File, Url), ()> {
    // Request Telegram file
    let file = request_telegram_file(file, state.clone()).await?;

    // Build URL
    file.get_url(state.token())
        .ok_or_else(|| {
            error!("No download URL for Telegram API file provided");
            ()
        })
        .and_then(|url| match Url::parse(&url) {
            Ok(url) => Ok((file, url)),
            Err(err) => {
                error!("Failed to parse file URL from Telegram API: {}", err);
                Err(())
            }
        })
}

/// Check whether the given Telegram image is an illegal file.
async fn is_illegal_image(config: &Scanner, file: File, url: &Url) -> bool {
    // Skip images that are too large
    match file.file_size {
        Some(size) if size > IMAGE_MAX_FILE_SIZE => {
            info!("Image file too large to audit, assuming safe");
            return false;
        },
        _ => {}
    };

    // Download the file to a temporary file to test on
    let path = match util::download::download_temp(url).await {
        Ok(response) => response.1,
        Err(err) => {
            warn!("Failed to download image file, could not audit, assuming safe: {:?}", err);
            return false;
        }
    };

    // Test whether the image file is illegal
    super::image::is_illegal_image(config, Arc::new(path)).await
}

/// Check whether the given Telegram video is an illegal file.
#[cfg(feature = "ffmpeg")]
async fn is_illegal_video(config: &Scanner, _: File, url: &Url) -> bool {
    // Download the file to a temporary file to test on
    let path = match util::download::download_temp(url).await {
        Ok(response) => response.1,
        Err(err) => {
            warn!("Failed to download video file, could not audit, assuming safe: {:?}", err);
            return false;
        }
    };

    // Extract video frames
    let frame_file = match util::video::extract_frames(Arc::new(path)).await {
        Ok(frame_file) => frame_file,
        Err(_) => {
            warn!("Failed to extract video frames, could not audit, assuming safe");
            return false;
        },
    };

    // Test whether the image file is illegal
    super::image::is_illegal_image(config, frame_file).await
}
