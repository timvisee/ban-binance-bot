[![Build status on GitLab CI][gitlab-ci-master-badge]][gitlab-ci-link]
[![Add bot on Telegram][telegram-badge]][telegram-link]

[gitlab-ci-link]: https://gitlab.com/timvisee/ban-binance-bot/pipelines
[gitlab-ci-master-badge]: https://gitlab.com/timvisee/ban-binance-bot/badges/master/pipeline.svg
[telegram-badge]: https://img.shields.io/badge/Telegram-@banbinancebot-blue.svg

_Note: This bot is a Work in Progress, it works but it's being tested with. The
source code will be a mess until testing is done and a first release is
published._

# Ban Binance Bot for Telegram [WIP]
A Telegram bot to ban users advertising Binance promotions.

![Binance spam screenshot](./res/binance-spam-screenshot-stop.png)

Public Telegram channels are infested with Binance spam these days, posted by
bots at random intervals. This bot counters this by scanning all messages with
images, files and links posted in group chats. If any illegal content (content
with Binance promotions) is posted, the user is instantly banned and their
message is removed.

## How does it work?
When the bot is added to a group, all content is scanned for Binance promotions.

The bot currently scans for illegal content, including:
- links going to Binance promotions
- files (pictures, GIFs, stickers, ...) containing a Binance promotion

When illegal content is found:
- the user is immediately banned from the chat
- the respective message is deleted
- users are notified in the group chat

No configuration needed, just add the bot to your group and it works out of the
box. Remove it again when you're done with it.

To unban a user, just manually add them to the group chat again.

Illegal messages by administrators may be deleted, but administrators are never banned.

## How to use?
I host a public instance of this bot which you can use in your own groups.

1.  Directly message [@banbinancebot][telegram-link] in Telegram,
    make sure the bot responds to verify it's still running.
2.  Add [@banbinancebot][telegram-link] to any of your Telegram
    groups.
3.  Make [@banbinancebot][telegram-link] administrator, to support
    automatic banning.
    - In normal groups; disable 'All Members Are Admins', mark the bot as administrator
    - In supergroups; mark the bot as administrator

You can always compile this bot yourself, to host your own instance.
In that case, please refer to this repository hosting the source code in the
description and about fields of your bot.

## Build & runtime requirements
Currently, Rust beta is used for the new async/await system for asynchronous
code.

Build requirements:
- Rust 1.39 (beta, or higher) (via [rustup](https://rustup.rs): `rustup default beta`)
- Feature specific:
  - `ocr`: (default, scan images for illegal text)
    - `tesseract`, `leptonica` libraries and `clang`:
      - Ubuntu: `sudo apt-get install libleptonica-dev libtesseract-dev clang`

Runtime requirements:
- A Telegram bot token
- Feature specific:
  - `ffmpeg`: (default, scan videos)
    - `ffmpeg`:
      - Ubuntu: `sudo apt-get install ffmpeg`
  - `ocr`: (default, scan images for illegal text)
    - `tesseract` data for English language:
      - Ubuntu: `sudo apt-get install tesseract-ocr-eng`

```bash
# Clone repository
git clone git@github.com:timvisee/ban-binance-bot.git
cd ban-binance-bot

# Use Rust beta in current directory
rustup override set beta

# Install build and runtime dependencies
sudo apt-get install libleptonica-dev libtesseract-dev tesseract-ocr-eng clang ffmpeg

# Build release
cargo build --release

# Configure .env
cp .env.example .env
nano .env

# Run the bot
./target/release/ban-binance-bot
```

To build the bot without some features, so you don't have to install extra
packages, use:

```bash
cargo build --release --no-default-features
```

The bot currently panics (crashes) for hard errors, such as connection errors to
the Telegram API. Properly handling these situations in code will be worked on
in the future. I therefore recommend running this bot with `supervisor` or in a
simple loop:

```bash
while true; do
  echo Starting bot...
  ./target/release/ban-binance-bot
  sleep 5
done
```

## Privacy notice
Once added to a group, this bot scans all following user messages to determine
whether illegal content is posted. All links are visited to determine whether
it links to any illegal content. All files (photos, GIFs, ...) are downloaded
and scanned for illegal content. All content is immediately deleted from the
machine the bot is running on after scanning. This is an automated process
with no user intervention.

Depending on runtime configuration for the bot, messages detected as spam may
be forwarded to a logging chat, including any false-positive detections, to
collect and monitor all spam. These messages will stay until the logging chat,
or the specific forwarded message, is deleted. Messages audited as safe are
never forwarded by this bot.

## License
This project is licensed under the GNU GPL-3.0 license.
Check out the [LICENSE](LICENSE) file for more information.

[telegram-link]: https://t.me/banbinancebot
