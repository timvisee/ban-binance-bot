// TODO: filter too large files, limit to 20MB
// TODO: filter too small images

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use dotenv::dotenv;
use dssim::{Dssim, ToRGBAPLU, RGBAPLU};
use futures::{
    future::{ok, result},
    prelude::*,
    stream::iter_ok,
    Future, Stream,
};
use image::{imageops, FilterType};
use image::{GenericImageView, Rgba};
use imgref::ImgVec;
use linkify::{LinkFinder, LinkKind};
use reqwest::{r#async::Client, Error as ResponseError, RedirectPolicy};
use rgb::RGBA;
use state::State;
use telegram_bot::{types::Message, *};
use tempfile::{Builder, TempPath};
use tokio_core::reactor::{Core, Handle};
use tokio_signal::ctrl_c;
use url::Url;

use traits::*;

mod state;
mod traits;

/// A list of illegal URL hosts.
const ILLEGAL_HOSTS: [&str; 3] = ["binance.mxevent.site", "mxevent.site", "binance.com"];

/// The maximum file size in bytes of files to check for legality.
const MAX_FILE_SIZE: i64 = 2 * 1024 * 1024;

/// Images are illegal when their similarity to any template image is `<= threhold`.
const IMAGE_BAN_THRESHOLD: f64 = 0.02;

fn main() {
    // Load the environment variables file
    dotenv().ok();

    // Build a future reactor
    let mut core = Core::new().unwrap();

    // Initialize the global state
    let state = State::init(core.handle());

    // Build a signal handling future to quit nicely
    let signal = ctrl_c()
        .flatten_stream()
        .into_future()
        .inspect(|_| eprintln!("Received CTRL+C signal, preparing to quit..."))
        .map(|_| ())
        .map_err(|_| ());

    // Build the application, attach signal handling
    let app = build_telegram_handler(state.clone(), core.handle())
        .select(signal)
        .map_err(|(e, _)| e)
        .then(|r| {
            eprintln!("Quitting...");
            result(r)
        });

    // Run the application future in the reactor
    core.run(app).unwrap();
}

/// Build a future for handling Telegram API updates.
fn build_telegram_handler(state: State, handle: Handle) -> impl Future<Item = (), Error = ()> {
    state
        .telegram_client()
        .stream()
        .for_each(move |update| {
            // Clone the state to get ownership
            let state = state.clone();

            // Process messages
            match update.kind {
                UpdateKind::Message(msg) => {
                    handle.spawn(handle_message(msg, state));
                }
                UpdateKind::EditedMessage(msg) => {
                    handle.spawn(handle_message(msg, state));
                }
                _ => {}
            }

            ok(())
        })
        .map_err(|err| {
            eprintln!("ERR: Telegram API updates loop error, ignoring: {}", err);
            ()
        })
}

/// Handle the given message.
///
/// This checks if the message is illegal, and immediately bans the sender if it is.
fn handle_message(msg: Message, state: State) -> Box<dyn Future<Item = (), Error = ()>> {
    // TODO: do not clone, but reference

    Box::new(is_illegal_message(msg.clone(), state.clone()).and_then(
        move |illegal| -> Box<dyn Future<Item = _, Error = _>> {
            // Ban users that send illegal messages
            if illegal {
                // Build the message, keep a reference to the chat
                let name = format_user_name(&msg.from);
                let chat = &msg.chat;

                // TODO: do not ignore error here
                let delete_msg = state.telegram_client().send(msg.delete()).map_err(|_| ());

                // TODO: do not ignore error here
                let notify_msg = state
                    .telegram_client()
                    .send(
                        chat.text(format!(
                            "Automatically banned {} for posing Binance promotions",
                            name,
                        ))
                        .parse_mode(ParseMode::Markdown)
                        .disable_preview()
                        .disable_notification(),
                    )
                    .map_err(|_| ());

                // TODO: do not ignore error here
                return Box::new(delete_msg.join(notify_msg).map(|_| ()));
            }

            Box::new(ok(()))
        },
    ))
}

/// Check whether the given message is illegal.
fn is_illegal_message(msg: Message, state: State) -> Box<dyn Future<Item = bool, Error = ()>> {
    // TODO: run check futures concurrently

    let mut future: Box<dyn Future<Item = _, Error = _>> = Box::new(ok(false));

    // Check message text
    if let Some(text) = msg.text() {
        future = Box::new(future.and_then(|_| is_illegal_text(text)));
    }

    // Check message files (pictures, stickers, files, ...)
    if let Some(files) = msg.files() {
        future = Box::new(
            future.and_then(|illegal| -> Box<dyn Future<Item = _, Error = _>> {
                if !illegal {
                    Box::new(has_illegal_files(files, state))
                } else {
                    Box::new(ok(illegal))
                }
            }),
        );
    }

    future
}

/// Check whether the given text is illegal.
fn is_illegal_text(text: String) -> impl Future<Item = bool, Error = ()> {
    // Check for illegal URLs
    contains_illegal_urls(&text)
}

/// Check whether any of the given files is illegal.
///
/// A list of `GetFile` requests is given, as the actual files should still be downloaded.
fn has_illegal_files(
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
fn is_illegal_file(file: GetFile, state: State) -> impl Future<Item = bool, Error = ()> {
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
        if url.ends_with(".jpg") || url.ends_with(".jpeg") || url.ends_with(".png") {
            return Box::new(download_temp(&url).and_then(|(_file, path)| is_illegal_image(&path)));
        }

        // TODO: remove after testing

        eprintln!("TODO: Test file at: {}", url);

        Box::new(ok(false))
    })
}

/// Download a file at the given URL to a temporary file on the system.
/// The downloaded file and path is returned.
///
/// The actual downloaded file is automatically deleted from disk when the last file handle
/// (`File`) is dropped. See `tempfile::NamedTempFile` for more details.
// TODO: make this properly async, the download process isn't at this moment
fn download_temp(url: &str) -> impl Future<Item = (File, TempPath), Error = ()> {
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

/// Check whether the given image is illegal.
fn is_illegal_image(image: &Path) -> impl Future<Item = bool, Error = ()> {
    eprintln!("Checking image...");

    // Load the images
    let base_image = image::open("./res/binance.jpg").expect("failed to open base");
    let image = image::open(image).expect("failed to open downloaded image");

    // Make the image we're testing the same size
    let (x, y) = base_image.dimensions();
    let image = imageops::resize(&image, x, y, FilterType::Triangle);

    // Create a DSSIM instance
    let mut dssim = Dssim::new();

    let base_image = to_imgvec(&base_image);
    let base_image = dssim
        .create_image(&base_image)
        .expect("failed to load base image");

    let image = to_imgvec(&image);
    let image = dssim.create_image(&image).expect("failed to load image");

    // Compare the images, obtain the score
    let result = dssim.compare(&base_image, image);
    let score = result.0;
    let is_similar = score <= IMAGE_BAN_THRESHOLD;

    if is_similar {
        println!("Illegal image! (score: {})", score);
    } else {
        println!("Allowed image (score: {})", score);
    }

    ok(is_similar)
}

fn to_imgvec(input: &impl GenericImageView<Pixel = Rgba<u8>>) -> ImgVec<RGBAPLU> {
    let pixels = input
        .pixels()
        .map(|(_x, _y, Rgba([r, g, b, a]))| RGBA::new(r, g, b, a))
        .collect::<Vec<_>>()
        .to_rgbaplu();
    ImgVec::new(pixels, input.width() as usize, input.height() as usize)
}

/// List all URLs in the given text.
fn find_urls(text: &str) -> Vec<Url> {
    // Set up the URL finder
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    // Collect all links, parse them to URL
    finder
        .links(text)
        .filter_map(|url| match Url::parse(url.as_str()) {
            Ok(url) => Some(url),
            Err(err) => {
                eprintln!("Failed to parse URL: {:?}", err);
                None
            }
        })
        .collect()
}

/// Check whether the given text contains any illegal URLs.
///
/// This uses `ILLEGAL_HOSTS`.
fn contains_illegal_urls(text: &str) -> Box<dyn Future<Item = bool, Error = ()>> {
    // TODO: do a forwarding check, compare target URLs as well

    // Find URLs in the message
    let urls = find_urls(text);

    // Scan for any static illegal URLs in the text message
    let illegal = urls.iter().any(is_illegal_url);
    if illegal {
        return Box::new(ok(true));
    }

    // Resolve all URL forwards, and test for illegal URLs again
    let future = iter_ok(urls)
        // Filter URLs that are still the same
        .and_then(|url| follow_url(&url))
        .and_then(|url| ok(is_illegal_url(&url)))
        // TODO: do not map errors here
        .into_future()
        // TODO: test all results here
        .map(|(result, _)| result.unwrap_or(false))
        .map_err(|_| ());

    // Follow redirects on all URLs, and test the target URLs again
    Box::new(future)
}

/// Check wheher the given URL is illegal.
fn is_illegal_url(url: &Url) -> bool {
    // Get the host
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    let host = host.trim().to_lowercase();

    ILLEGAL_HOSTS
        .into_iter()
        .any(|illegal_host| illegal_host == &host)
}

/// Follow redirects on the given URL, and return the final full URL.
///
/// This is used to obtain share URLs from shortened links.
///
// TODO: extract this into module
pub fn follow_url(url: &Url) -> impl Future<Item = Url, Error = FollowError> {
    // Build the URL client
    // TODO: use a global client instance
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(RedirectPolicy::limited(25))
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(60))
        .build()
        .expect("failed to build URL forward resolver client");

    println!("Checking URL for redirects: {}", url.as_str());

    // Send the request, follow the URL, ensure success
    let response = client
        .get(url.as_str())
        .send()
        .map_err(FollowError::Request);

    // TODO: validate status !response.status.is_success()

    // Obtain the final URL
    response.map(|r| r.url().clone())
}

/// URL following error.
#[derive(Debug)]
pub enum FollowError {
    /// Failed to send the shortening request.
    // #[fail(display = "failed to send URL follow request")]
    Request(reqwest::Error),

    /// The server responded with a bad response.
    // #[fail(display = "failed to shorten URL, got bad response")]
    Response(ResponseError),
}

impl From<ResponseError> for FollowError {
    fn from(err: ResponseError) -> Self {
        FollowError::Response(err)
    }
}

/// Format the name of a given Telegram user.
///
/// The output consists of:
/// - A first name
/// - A last name (if known)
/// - Clickable name (if username is known)
///
/// The returned string should be sent with `.parse_mode(ParseMode::Markdown)` enabled.
fn format_user_name(user: &User) -> String {
    // Take the first name
    let mut name = user.first_name.clone();

    // Append the last name if known
    if let Some(last_name) = &user.last_name {
        name.push_str(" ");
        name.push_str(last_name);
    }

    // Make clickable if username is known
    if let Some(username) = &user.username {
        name.insert(0, '[');
        name.push_str(&format!("](https://t.me/{})", username));
    }

    name
}
