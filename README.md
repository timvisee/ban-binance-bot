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

### Privacy notice
Once added to a group, this bot scans all following user messages to determine
whether illegal content is posted. All links are visited to determine whether
the link to any illegal content. All files (photos, GIFs, ...) are downloaded
and scanned for illegal content. All content is immediately deleted from the
servers disk the bot is running on after scanning. This is an automated process
with no user intervention.

## License
This project is licensed under the GNU GPL-3.0 license.
Check out the [LICENSE](LICENSE) file for more information.

[telegram-link]: https://t.me/banbinancebot
