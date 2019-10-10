#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use dotenv::dotenv;
use state::State;

mod bot;
mod config;
mod scanner;
mod state;
mod util;

use bot::UpdateError;

#[tokio::main(multi_thread)]
async fn main() -> Result<(), UpdateError> {
    // Load the environment variables file
    dotenv().ok();

    // Enable logging
    env_logger::init();

    info!("Starting Telegram bot...");

    // Initialize the global state
    let state = match State::init().await {
        Ok(state) => state,
        Err(err) => {
            error!("Failed to initialize bot state: {}", err);
            panic!();
        },
    };
    debug!("Bot has been initialized");

    // Build the application, attach signal handling
    let app = bot::build_telegram_handler(state.clone()).await;
    match &app {
        Ok(_) => info!("Quit successfully"),
        Err(err) => error!("Quit with error!\n{:?}", err),
    }

    app
}
