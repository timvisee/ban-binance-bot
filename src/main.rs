use dotenv::dotenv;
use futures::{
    future::{ok, result},
    Future, Stream,
};
use linkify::{LinkFinder, LinkKind};
use state::State;
use telegram_bot::{
    types::{Message},
    *,
};
use tokio_core::reactor::{Core, Handle};
use tokio_signal::ctrl_c;
use url::Url;

use traits::*;

mod state;
mod traits;

/// A list of illegal URL hosts.
const ILLEGAL_HOSTS: [&str; 2] = [
    "binance.mxevent.site",
    "mxevent.site",
];

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
                UpdateKind::Message(message) => {
                    // TODO: check this message

                    let future = is_illegal_message(message)
                        .and_then(|illegal| {
                            println!("Got illegal message: {:?}", illegal);
                            ok(())
                        });
                    handle.spawn(future);
                }
                UpdateKind::EditedMessage(message) => {
                    // TODO: check this message

                    let future = is_illegal_message(message)
                        .and_then(|illegal| {
                            println!("Got illegal edited message: {:?}", illegal);
                            ok(())
                        });
                    handle.spawn(future);
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

/// Check whether the given message is illegal.
fn is_illegal_message(msg: Message) -> Box<dyn Future<Item = bool, Error = ()>> {
    // TODO: run check futures concurrently

    let mut future: Box<dyn Future<Item = _, Error = _>> = Box::new(ok(false));

    // Check message text
    if let Some(text) = msg.text() {
        future = Box::new(future.and_then(|_| is_illegal_text(text)));
    }

    // Check for any images
    future = Box::new(future.and_then(|illegal| -> Box<Future<Item = _, Error = _>> {
        if !illegal {
            Box::new(is_illegal_image(msg))
        } else {
            Box::new(ok(illegal))
        }
    }));

    future
}

/// Check whether the given text is illegal.
fn is_illegal_text(text: String) -> impl Future<Item = bool, Error = ()> {
    // Check for illegal URLs
    if contains_illegal_urls(&text) {
        println!("Message contained illegal URL");
        return ok(true);
    }

    // TODO: return proper answer here
    ok(false)
}

/// Check whether the given image is illegal.
fn is_illegal_image(image: Message) -> impl Future<Item = bool, Error = ()> {
    // TODO: return proper answer here
    ok(false)
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
            },
        })
        .collect()
}

/// Check whether the given text contains any illegal URLs.
///
/// This uses `ILLEGAL_HOSTS`.
fn contains_illegal_urls(text: &str) -> bool {
    find_urls(text)
        .into_iter()
        .any(|url| {
            // Get the host
            let host = match url.host_str() {
                Some(host) => host,
                None => return false,
            };
            let host = host.trim().to_lowercase();

            ILLEGAL_HOSTS.into_iter().any(|illegal_host| illegal_host == &host)
        })
}
