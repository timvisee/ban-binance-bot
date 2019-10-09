#[macro_use]
extern crate lazy_static;

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

    // Initialize the global state
    let state = State::init();

    // Build the application, attach signal handling
    let app = bot::build_telegram_handler(state.clone()).await;
    match &app {
        Ok(_) => println!("Bot quit successfully"),
        Err(err) => println!("Bot quit with error!\n{:?}", err),
    }

    app
}
