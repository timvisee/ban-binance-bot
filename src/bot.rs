use futures::prelude::*;
use telegram_bot::{types::Message, *};

use crate::{scanner, state::State, traits::*, util};

/// Build a future for handling Telegram API updates.
// TODO: handle update errors here
pub async fn build_telegram_handler(state: State) -> Result<(), ()> {
    // Fetch new updates via long poll method
    let mut stream = state.telegram_client().stream();

    while let Some(update) = stream.next().await {
        // Make sure we received a enw update
        // TODO: do not drop error here
        let update = update.map_err(|_| ())?;

        // Process messages
        match update.kind {
            UpdateKind::Message(msg) => match &msg.chat {
                MessageChat::Private(..) => {
                    handle_private(&state, &msg).await?;
                },
                _ => {handle_message(msg, state.clone()).await?;},
            },
            UpdateKind::EditedMessage(msg) => {
                handle_message(msg, state.clone()).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handle the given private/direct message.
///
/// This simply notifies the user that the bot is active, and doesn't really do anything else.
async fn handle_private(state: &State, msg: &Message) -> Result<(), ()> {
    // Log that we're receiving a private message
    println!(
        "Received private message from {}: {}",
        util::telegram::format_user_name_log(&msg.from),
        msg.text().unwrap_or_else(|| "?".into())
    );

    state
        .telegram_client()
        .send(
            msg.text_reply(format!(
                "`BLEEP BLOOP`\n`I AM A BOT`\n\n\
                {}, add me to a group to start banning Binance advertising bots.\n\n\
                [» How does it work?](https://github.com/timvisee/ban-binance-bot/blob/master/README.md#how-does-it-work)\n\
                [» How to use?](https://github.com/timvisee/ban-binance-bot/blob/master/README.md#how-to-use)",
                msg.from.first_name,
            ))
            .parse_mode(ParseMode::Markdown)
            .disable_preview(),
        )
        // TODO: do not drop error here
        .map(|_| Ok(()))
        .await
}

/// Handle the given message.
///
/// This checks if the message is illegal, and immediately bans the sender if it is.
async fn handle_message(msg: Message, state: State) -> Result<(), ()> {
    // Return if not illegal, ban user otherwise
    if !is_illegal_message(msg.clone(), state.clone()).await? {
        return Ok(());
    }

    // Build the message, keep a reference to the chat
    let name = util::telegram::format_user_name(&msg.from);
    let chat = &msg.chat;

    // Attempt to kick the user, and delete their message
    let kick_user = state.telegram_client().send(msg.from.kick_from(&chat)).await;
    let _ = state.telegram_client().send(msg.delete()).await;

    // Build the notification to share in the chat
    let notification = if kick_user.is_err() {
        format!(
            "An administrator should ban {} for posting Binance promotions.\n\n\
            [Add](https://github.com/timvisee/ban-binance-bot/blob/master/README.md#how-to-use) this bot as explicit administrator in this group to automatically ban users posting new promotions. \
            Administrators are never banned automatically.",
            name,
        )
    } else {
        format!(
            "Automatically banned {} for posting Binance promotions.",
            name,
        )
    };

    // Attempt to send a ban notification to the chat
    let _ = state
        .telegram_client()
        .send(
            chat.text(notification)
                .parse_mode(ParseMode::Markdown)
                .disable_preview()
                .disable_notification(),
        )
        .map_err(|err| {
            eprintln!("Failed to send ban notification in chat, ignoring...\n{:?}", err);
            ()
        })
        .await;

    Ok(())
}

/// Check whether the given message is illegal.
async fn is_illegal_message(msg: Message, state: State) -> Result<bool, ()> {
    // TODO: run check futures concurrently

    // Check message text
    if let Some(text) = msg.text() {
        if scanner::text::is_illegal_text(text).await? {
            return Ok(true);
        }
    }

    // Check message files (pictures, stickers, files, ...)
    if let Some(files) = msg.files() {
        if scanner::file::has_illegal_files(files, state).await? {
            return Ok(true);
        }
    }

    Ok(false)
}
