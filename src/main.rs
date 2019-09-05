use dotenv::dotenv;
use futures::{
    future::{ok, result},
    Future, Stream,
};
use state::State;
use telegram_bot::*;
use tokio_core::reactor::{Core, Handle};
use tokio_signal::ctrl_c;

use traits::*;

mod state;
mod traits;

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
    let app = build_application(state.clone(), core.handle())
        .select(signal)
        .map_err(|(e, _)| e)
        .then(|r| {
            eprintln!("Quitting...");
            result(r)
        });

    // Run the application future in the reactor
    core.run(app).unwrap();
}

/// Build the future for running the main application, which is the bot.
fn build_application(state: State, handle: Handle) -> impl Future<Item = (), Error = ()> {
    let telegram = build_telegram_handler(state, handle);
    telegram.map(|_| ())
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

                    let text = message.text();
                    println!("MSG: {:?}", text);
                }
                UpdateKind::EditedMessage(message) => {
                    // TODO: check this message

                    let text = message.text();
                    println!("EDITED MSG: {:?}", text);
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
