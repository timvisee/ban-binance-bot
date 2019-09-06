// TODO: filter too large files, limit to 20MB
// TODO: filter too small images

use dotenv::dotenv;
use futures::{
    future::{ok, result},
    Future, Stream,
};
use state::State;
use telegram_bot::{types::Message, *};
use tokio_core::reactor::{Core, Handle};
use tokio_signal::ctrl_c;
use traits::*;

mod config;
mod scanner;
mod state;
mod traits;
mod util;

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

    // Run the Telegram bot logic future in the reactor
    let _ = core
        .run(app)
        .expect("an error occurred while running Telegram bot update loop");
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
                UpdateKind::Message(msg) => match &msg.chat {
                    MessageChat::Private(..) => handle.spawn(handle_private(&state, &msg)),
                    _ => handle.spawn(handle_message(msg, state)),
                },
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

/// Handle the given private/direct message.
///
/// This simply notifies the user that the bot is active, and doesn't really do anything else.
fn handle_private(state: &State, msg: &Message) -> Box<dyn Future<Item = (), Error = ()>> {
    Box::new(
        state
            .telegram_client()
            .send(
                msg.text_reply(format!(
                    "`BLEEP BLOOP`\n`I AM A BOT`\n\n{}, add me to a group to start banning Binance promotion bots.",
                    msg.from.first_name,
                ))
                .parse_mode(ParseMode::Markdown),
            )
            .map(|_| ())
            .map_err(|_| ()),
    )
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
                let name = util::telegram::format_user_name(&msg.from);
                let chat = msg.chat.clone();

                // TODO: do not ignore error here
                let kick_user = state.telegram_client().send(msg.from.kick_from(&chat));

                let future = kick_user.then(move |result| {
                    // Check whether we failed to delete
                    let failed = result.is_err();

                    // TODO: do not ignore error here
                    let delete_msg = state.telegram_client().send(msg.delete()).map_err(|_| ());

                    // Build the notification to share in the chat
                    let notification = if failed {
                        format!(
                            "An administrator should ban {} for posing Binance promotions.\n\n\
                            Add this bot as explicit administrator in this group to automatically ban users posting new promotions. \
                            Administrators are never banned automatically.",
                            name,
                        )
                    } else {
                        format!(
                            "Automatically banned {} for posing Binance promotions.",
                            name,
                        )
                    };

                    // TODO: do not ignore error here
                    let notify_msg = state
                        .telegram_client()
                        .send(
                            chat.text(notification)
                                .parse_mode(ParseMode::Markdown)
                                .disable_preview()
                                .disable_notification(),
                        )
                        .map_err(|_| ());

                    delete_msg.join(notify_msg).map(|_| ())
                });

                // TODO: do not ignore error here
                return Box::new(future);
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
        future = Box::new(future.and_then(|_| scanner::text::is_illegal_text(text)));
    }

    // Check message files (pictures, stickers, files, ...)
    if let Some(files) = msg.files() {
        future = Box::new(
            future.and_then(|illegal| -> Box<dyn Future<Item = _, Error = _>> {
                if !illegal {
                    Box::new(scanner::file::has_illegal_files(files, state))
                } else {
                    Box::new(ok(illegal))
                }
            }),
        );
    }

    future
}
