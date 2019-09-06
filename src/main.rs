// TODO: filter too large files, limit to 20MB
// TODO: filter too small images

use dotenv::dotenv;
use futures::{
    future::result,
    Future, Stream,
};
use state::State;
use tokio_core::reactor::Core;
use tokio_signal::ctrl_c;

mod bot;
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
    let app = bot::build_telegram_handler(state.clone(), core.handle())
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
