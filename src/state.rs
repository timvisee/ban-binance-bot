use std::env;

use telegram_bot::Api;

/// The global application state.
#[derive(Clone)]
pub struct State {
    /// The Telegram API bot token.
    token: String,

    /// The Telegram API client beign used.
    telegram_client: Api,
}

impl State {
    /// Initialize.
    ///
    /// This initializes the global state.
    /// Internally this creates the Telegram API client and sets up a connection,
    /// connects to the bot database and more.
    ///
    /// A handle to the Tokio core reactor must be given to `reactor`.
    pub fn init() -> State {
        // Retrieve the Telegram bot token
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("env var TELEGRAM_BOT_TOKEN not set");

        State {
            telegram_client: Self::create_telegram_client(&token),
            token,
        }
    }

    /// Create a Telegram API client instance, and initiate a connection.
    fn create_telegram_client(token: &str) -> Api {
        // Initiate the Telegram API client
        Api::new(token)
    }

    /// Get the Telegram API client.
    pub fn telegram_client(&self) -> &Api {
        &self.telegram_client
    }

    /// Get the Telegram bot token.
    pub fn token(&self) -> &str {
        &self.token
    }
}
