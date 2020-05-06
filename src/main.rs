#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

#[cfg(feature = "sentry")]
use std::env;

use dotenv::dotenv;

use state::State;

mod bot;
mod config;
mod scanner;
mod state;
mod traits;
mod util;

use bot::UpdateError;

#[tokio::main(multi_thread)]
async fn main() -> Result<(), UpdateError> {
    // Load the environment variables file
    dotenv().ok();

    // Enable logging
    env_logger::init();

    // Initialize Sentry
    #[cfg(feature = "sentry")]
    let _guard = init_sentry();

    info!("Starting Telegram bot...");

    // Initialize the global state
    let state = match State::init().await {
        Ok(state) => state,
        Err(err) => {
            error!("Failed to initialize bot state: {:?}", err);
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

/// Initialize Sentry.
///
/// This returns a guard that must be kept.
#[cfg(feature = "sentry")]
fn init_sentry() -> Option<sentry::internals::ClientInitGuard> {
    // Get the sentry DNS string
    let dns = match env::var("SENTRY_DNS") {
        Ok(dns) if !dns.is_empty() => dns,
        Ok(_) | Err(_) => {
            info!("Not enabling Sentry, no Sentry DNS configured");
            return None;
        },
    };

    // Initialize Sentry, register some handlers
    let guard = sentry::init(dns);
    sentry::integrations::panic::register_panic_handler();

    debug!("Initialized Sentry");

    Some(guard)
}
