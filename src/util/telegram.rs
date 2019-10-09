use telegram_bot::{User, MessageChat};

/// Format the name of a given Telegram user.
///
/// The output consists of:
/// - A first name
/// - A last name (if known)
/// - Clickable name (if username is known)
///
/// The returned string should be sent with `.parse_mode(ParseMode::Markdown)` enabled.
pub fn format_user_name(user: &User) -> String {
    // Take the first name
    let mut name = user.first_name.clone();

    // Append the last name if known
    if let Some(last_name) = &user.last_name {
        name.push_str(" ");
        name.push_str(last_name);
    }

    // Make clickable if username is known
    if let Some(username) = &user.username {
        name.insert(0, '[');
        name.push_str(&format!("](https://t.me/{})", username));
    }

    name
}

/// Format the name of a given Telegram user for logging.
///
/// The output consists of:
/// - A first name
/// - A last name (if known)
/// - Clickable name (if username is known)
pub fn format_user_name_log(user: &User) -> String {
    // Take the first name
    let mut name = user.first_name.clone();

    // Append the last name if known
    if let Some(last_name) = &user.last_name {
        name.push_str(" ");
        name.push_str(last_name);
    }

    // Make clickable if username is known
    if let Some(username) = &user.username {
        name = format!("@{} ({})", username, name);
    }

    name
}

/// Format the name of a given Telegram chat.
///
/// The output consists of:
/// - Group name
/// - Clickable name (if handle is known)
/// - Group ID (if handle is not know)
///
/// The returned string should be sent with `.parse_mode(ParseMode::Markdown)` enabled.
pub fn format_chat_name(chat: &MessageChat) -> String {
    match chat {
        MessageChat::Private(user) => {
            format!("{} (direct message)", format_user_name(user))
        },
        MessageChat::Group(group) => {
            format!("'_{}_' (`{}`)", group.title, group.id)
        },
        MessageChat::Supergroup(group) => {
            match &group.username {
                Some(handle) => format!("[{}](https://t.me/{})", group.title, handle),
                None => format!("'_{}_' (`{}`)", group.title, group.id),
            }
        },
        MessageChat::Unknown(_) => "?".into(),
    }
}

/// Format the name of a given Telegram chat for logging.
///
/// The output consists of:
/// - Group name
/// - Group handle (if handle is known)
/// - Group ID (if handle is not know)
pub fn format_chat_name_log(chat: &MessageChat) -> String {
    match chat {
        MessageChat::Private(user) => {
            format!("{} (direct message)", format_user_name_log(user))
        },
        MessageChat::Group(group) => {
            format!("'{}' ({})", group.title, group.id)
        },
        MessageChat::Supergroup(group) => {
            match &group.username {
                Some(handle) => format!("@{} ({})", handle, group.title),
                None => format!("'{}' ({})", group.title, group.id),
            }
        },
        MessageChat::Unknown(_) => "?".into(),
    }
}
