use std::env;

use telegram_bot::{
    Api,
    Error as TelegramError,
    types::{GetMe, User},
};

/// The global application state.
#[derive(Clone)]
pub struct State {
    /// The Telegram API bot token.
    token: String,

    /// The Telegram API client beign used.
    telegram_client: Api,

    /// The bot user.
    user: User,
}

impl State {
    /// Initialize.
    ///
    /// This initializes the global state.
    /// Internally this creates the Telegram API client and sets up a connection,
    /// connects to the bot database and more.
    ///
    /// A handle to the Tokio core reactor must be given to `reactor`.
    pub async fn init() -> Result<State, TelegramError> {
        // Retrieve the Telegram bot token
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("env var TELEGRAM_BOT_TOKEN not set");

        // Build the Telegram API
        let telegram_client = Self::create_telegram_client(&token);

        // Request bot user details
        let user = match telegram_client.send(GetMe).await {
            Ok(user) => {
                info!(
                    "Received bot details via Telegram API (user: {})",
                    user.username.as_ref().map(|u| u.as_str()).unwrap_or("?"),
                );
                user
            },
            Err(err) => {
                error!("Failed to request bot details via Telegram API: {}", err);
                return Err(err);
            },
        };

        Ok(State { telegram_client, token, user })
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

    /// Check whether the given user is this bot.
    pub fn is_bot_user(&self, user: &User) -> bool {
        self.user.id == user.id && user.is_bot
    }
}
