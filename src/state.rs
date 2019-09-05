use std::env;
use std::rc::Rc;

use telegram_bot::Api;
use tokio_core::reactor::Handle;

/// The global application state.
#[derive(Clone)]
pub struct State {
    /// The Telegram API bot token.
    token: String,

    /// The Telegram API client beign used.
    telegram_client: Api,

    /// The inner state.
    inner: Rc<StateInner>,
}

impl State {
    /// Initialize.
    ///
    /// This initializes the global state.
    /// Internally this creates the Telegram API client and sets up a connection,
    /// connects to the bot database and more.
    ///
    /// A handle to the Tokio core reactor must be given to `reactor`.
    pub fn init(reactor: Handle) -> State {
        // Retrieve the Telegram bot token
        let token = env::var("TELEGRAM_BOT_TOKEN").expect("env var TELEGRAM_BOT_TOKEN not set");

        State {
            telegram_client: Self::create_telegram_client(&token, reactor.clone()),
            token,
            inner: Rc::new(StateInner::init(reactor)),
        }
    }

    /// Create a Telegram API client instance, and initiate a connection.
    fn create_telegram_client(token: &str, reactor: Handle) -> Api {
        // Initiate the Telegram API client, and return
        Api::configure(token)
            .build(reactor)
            .expect("failed to initialize Telegram API client")
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

/// The inner state.
struct StateInner {
    /// A handle to the reactor.
    _handle: Handle,
}

impl StateInner {
    /// Initialize.
    ///
    /// This initializes the inner state.
    /// Internally this connects to the bot database.
    pub fn init(handle: Handle) -> StateInner {
        StateInner { _handle: handle }
    }
}
