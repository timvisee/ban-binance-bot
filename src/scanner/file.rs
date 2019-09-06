use std::time::Duration;

use futures::{future::ok, stream::iter_ok, Future, Stream};
use telegram_bot::*;

use crate::{config::*, state::State, util};

/// Check whether any of the given files is illegal.
///
/// A list of `GetFile` requests is given, as the actual files should still be downloaded.
pub fn has_illegal_files(
    mut files: Vec<GetFile>,
    state: State,
) -> impl Future<Item = bool, Error = ()> {
    // TODO: reverse list of files here (pick biggest image first)?
    files.reverse();

    // Test all files in order, return if any is illegal
    iter_ok(files)
        // TODO: do not clone state here
        .and_then(move |file| is_illegal_file(file, state.clone()))
        .filter(|illegal| *illegal)
        .into_future()
        .map(|(illegal, _)| match illegal {
            Some(illegal) => illegal,
            None => false,
        })
        .map_err(|(err, _)| err)
}

/// Check whether the given file is illegal.
///
/// A `GetFile` request is given, as the actual file should still be downloaded.
pub fn is_illegal_file(file: GetFile, state: State) -> impl Future<Item = bool, Error = ()> {
    // Request the file from Telegram
    let file_url = state
        .telegram_client()
        .send_timeout(file, Duration::from_secs(30))
        // TODO: do not ignore error here
        .map_err(|_| ())
        .and_then(|file| file.ok_or(()))
        .map(move |file| {
            // TODO: do not error here
            let url = file.get_url(state.token()).expect("failed to get file URL");
            (file, url)
        });

    // Test the file
    file_url.and_then(|(file, url)| -> Box<dyn Future<Item = _, Error = _>> {
        // Skip files that are too large
        match file.file_size {
            Some(size) if size > MAX_FILE_SIZE => return Box::new(ok(false)),
            _ => {}
        }

        // TODO: better extension test
        if url.ends_with(".jpg")
            || url.ends_with(".jpeg")
            || url.ends_with(".png")
            || url.ends_with(".webp")
        {
            return Box::new(
                util::download::download_temp(&url)
                    .and_then(|(_file, path)| super::image::is_illegal_image(&path)),
            );
        }

        // TODO: remove after testing

        eprintln!("TODO: Test file at: {}", url);

        Box::new(ok(false))
    })
}
