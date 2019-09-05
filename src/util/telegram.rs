use telegram_bot::User;

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
