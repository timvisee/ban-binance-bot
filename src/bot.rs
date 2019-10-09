use std::env;
use std::time::Duration;

use futures::prelude::*;
use telegram_bot::{prelude::*, types::{ChatId, Message, Update, MessageKind, MessageChat, UpdateKind, ParseMode}, Error as TelegramError};
use tokio::timer::delay_for;
use took::Timer;

use crate::{
    config::*,
    scanner,
    state::State,
    util::{self, future::select_true},
};

/// Build a future for handling Telegram API updates.
pub async fn build_telegram_handler(state: State) -> Result<(), UpdateError> {
    // Fetch new updates via long poll method, buffer to handle updates concurrently
    let mut stream = state
        .telegram_client()
        .stream()
        .map(|update| handle_update(state.clone(), update))
        .buffer_unordered(*TELEGRAM_CONCURRENT_UPDATES);

    // Run the update stream to completion
    while let Some(update) = stream.next().await {
        // Return errors
        if update.is_err() {
            return update;
        }
    }

    Ok(())
}

/// Handle the given Telegram API update.
async fn handle_update(state: State, update: Result<Update, TelegramError>) -> Result<(), UpdateError> {
    // Make sure we received a enw update
    // TODO: do not drop error here
    let update = update.map_err(UpdateError::Telegram)?;

    // Process messages
    match update.kind {
        UpdateKind::Message(msg) => match &msg.chat {
            MessageChat::Private(..) => handle_private(&state, &msg).await?,
            _ =>  handle_message(msg, state.clone()).await?,
        },
        UpdateKind::EditedMessage(msg) => handle_message(msg, state.clone()).await?,
        _ => {}
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

    // Define the status text
    let status_text = format!(
        "`BLEEP BLOOP`\n`I AM A BOT`\n\n\
        {}, add me to a group to start banning Binance advertising bots.\n\n\
        [» How does it work?](https://github.com/timvisee/ban-binance-bot/blob/master/README.md#how-does-it-work)\n\
        [» How to use?](https://github.com/timvisee/ban-binance-bot/blob/master/README.md#how-to-use)",
        msg.from.first_name,
    );

    // Post the status message
    let status = state
        .telegram_client()
        .send(
            msg.text_reply(format!("{}\n\n_Auditing your message..._", status_text))
                .parse_mode(ParseMode::Markdown)
                .disable_preview(),
        )
        // TODO: do not drop error here
        .map_err(|_| ())
        .await?;

    // Test message for legality, and build legality text
    let timer = Timer::new();
    let illegal = is_illegal_message(msg.clone(), state.clone()).await;
    let took = timer.took();
    let legality_text = if illegal {
        format!("_Unsafe! Your message is considered unsafe as it seems to contain Binance spam!\nThe message would be deleted automatically by this bot in groups the bot is added in._")
    } else {
        format!("_Safe. Your message is considered safe, and is not seen as Binance spam.\nSend me something else to test._")
    };

    // Post a generic direct message status
    state
        .telegram_client()
        .send(
            status
                .edit_text(format!(
                    "{}\n\n*Message audit:*\n{}\n\n_Audit took {}._",
                    status_text, legality_text, took
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
    let timer = Timer::new();
    if !is_illegal_message(msg.clone(), state.clone()).await {
        return Ok(());
    }
    let took = timer.took();

    println!(
        "Banning {} in {} for spam, audit took {}",
        util::telegram::format_user_name_log(&msg.from),
        util::telegram::format_chat_name_log(&msg.chat),
        took,
    );

    // Build the message, keep a reference to the chat
    let name = util::telegram::format_user_name(&msg.from);
    let chat = &msg.chat;

    // Attempt to kick the user, and delete their message
    let kick_user = state
        .telegram_client()
        .send(msg.from.kick_from(&chat))
        .await;

    // Forward the message to the global spam log chat
    let mut forward_msg = None;
    let forward_chat_id = env::var("GLOBAL_SPAM_LOG_CHAT_ID").ok().and_then(|id| id.parse().ok());
    if let Some(id) = forward_chat_id {
        // Do not forward if in same chat
        let id = ChatId::new(id);
        if msg.chat.id() == id {
            println!("Not forwarding spam to global log chat, is same chat");
        } else {
            // Forward
            match state.telegram_client().send(msg.forward(id)).await {
                Ok(msg) => forward_msg = Some(msg),
                Err(err) => println!("Failed to forward spam message to global log chat, ignoring: {:?}", err),
            }
        }
    }

    // Delete the user message
    let delete = state.telegram_client().send(msg.delete()).await;
    if let Err(err) = &delete {
        println!("Failed to delete spam message, might not have enough permission, ignoring: {}", err);
    }

    // Build the notification to share in the chat
    let mut notification = if kick_user.is_err() {
        format!(
            "An admin should ban {} for posting spam/phishing.{}\n\n\
            [Add](https://github.com/timvisee/ban-binance-bot/blob/master/README.md#how-to-use) this bot as explicit administrator to automatically ban users posting new promotions. \
            Administrators are never banned.",
            name,
            if delete.is_ok() {
                " I've deleted the message."
            } else {
                ""
            }
        )
    } else {
        format!(
            "Automatically banned {} for posting spam/phishing.",
            name,
        )
    };

    // Add self-destruct notice
    let self_destruct = NOTIFY_SELF_DESTRUCT_TIME.is_some();
    if self_destruct {
        notification += &format!("\n\n_This message will self-destruct in {} seconds..._", NOTIFY_SELF_DESTRUCT_TIME.unwrap());
    }

    // Attempt to send a ban notification to the chat
    let notify_msg = state
        .telegram_client()
        .send(
            // TODO: only show self destruct if actually self destructing
            // TODO: make time configurable
            chat.text(notification)
                .parse_mode(ParseMode::Markdown)
                .disable_preview()
                .disable_notification(),
        )
        .inspect_err(|err| println!(
            "Failed to send ban notification in chat, ignoring...\n{:?}",
            err
        ))
        .await;

    // Annotate forwarded spam message
    if let Some(forward_msg) = forward_msg {
        let state = state.clone();
        let mut annotate = forward_msg.text_reply(
            format!(
                "Banned this message in {} from {}.\n\n_Audit took {}._",
                util::telegram::format_chat_name(&msg.chat),
                util::telegram::format_user_name(&msg.from),
                took,
            )
        );

        tokio::spawn(async move {
            // Wait, prevent throttling, then annotate the forwarded spam
            delay_for(Duration::from_secs(2)).await;
            state.telegram_client().send(annotate
                    .parse_mode(ParseMode::Markdown)
                    .disable_preview()
                    .disable_notification()
                )
                .inspect_err(|err| println!("Failed to annotate forwarded spam message, ignoring...\n{:?}", err))
                .map(|_| ())
                .await
        });
    }

    // Self-destruct messages
    if self_destruct {
        if let Ok(msg) = notify_msg {
            tokio::spawn(async move {
                // Wait, then self destruct the message
                delay_for(Duration::from_secs(NOTIFY_SELF_DESTRUCT_TIME.unwrap())).await;
                state.telegram_client().send(msg.delete())
                    .inspect_err(|err| println!("Failed to self destruct ban notification, ignoring...\n{:?}", err))
                    .map(|_| ())
                    .await
            });
        }
    }

    Ok(())
}

/// Check whether the given message is illegal.
async fn is_illegal_message(msg: Message, state: State) -> bool {
    let mut checks = vec![];

    // Check message text
    if let Some(text) = msg.text() {
        // Scan any hidden URLs
        match &msg.kind {
            MessageKind::Text { entities, .. } =>  {
                let urls = util::url::find_hidden_urls(entities);
                if !urls.is_empty() {
                    checks.push(scanner::url::any_illegal_url(urls).boxed());
                }
            }
            _ => {},
        }

        // Scan the regular text
        checks.push(scanner::text::is_illegal_text(text).boxed());
    }

    // Check message files (pictures, stickers, files, ...)
    if let Some(files) = msg.get_files() {
        checks.push(scanner::file::has_illegal_files(files, state).boxed());
    }

    select_true(checks).await
}

/// The update error kind.
#[derive(Debug)]
pub enum UpdateError {
    /// An error occurred in the Telegram API.
    Telegram(TelegramError),

    /// An other update occurred.
    Other,
}

impl From<()> for UpdateError {
    fn from(_: ()) -> UpdateError {
        UpdateError::Other
    }
}
