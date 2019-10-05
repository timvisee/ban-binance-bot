// TODO: filter too large files, limit to 20MB
// TODO: filter too small images

use dotenv::dotenv;
use state::State;

mod bot;
mod config;
mod scanner;
mod state;
mod traits;
mod util;

#[tokio::main]
async fn main() -> Result<(), ()> {
    // Load the environment variables file
    dotenv().ok();

    // Initialize the global state
    let state = State::init();

    // Build the application, attach signal handling
    let app = bot::build_telegram_handler(state.clone()).await;
    match app {
        Ok(_) => eprintln!("Bot quit successfully"),
        Err(err) => eprintln!("Bot quit with error!\n{:?}", err),
    }

    app
}
