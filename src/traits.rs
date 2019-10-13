use telegram_bot::types::MessageChat;

pub trait ChatUsername {
    /// Get the username of a chat if available.
    ///
    /// This username or handle can be used to reference the chat.
    /// Returns `None` if the chat has no username configured.
    /// Direct chats do return `None` because theres nothing visible to others.
    fn username(&self) -> Option<String>;

    /// Get a link to the chat by it's username.
    ///
    /// Returns `None` if the chat has no username configured.
    /// Direct chats do return `None` because theres nothing visible to others.
    fn username_link(&self) -> Option<String> {
        self.username().map(|u| format!("https://t.me/{}", u))
    }
}

impl ChatUsername for MessageChat {
    fn username(&self) -> Option<String> {
        match self {
            MessageChat::Supergroup(group) => group.username.clone(),
            _ => None,
        }
    }
}
