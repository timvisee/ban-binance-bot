use telegram_bot::{
    types::{
        requests::get_file::GetFile,
        ChannelPost,
        Message,
        MessageKind,
        MessageOrChannelPost,
    },
    prelude::CanGetFile,
};

/// A trait to obtain text from a message.
pub trait MessageText {
    /// Obtain text from a message if available.
    fn text<'a>(&'a self) -> Option<String>;
}

impl MessageText for MessageOrChannelPost {
    fn text<'a>(&'a self) -> Option<String> {
        match self {
            MessageOrChannelPost::Message(msg) => msg.text(),
            MessageOrChannelPost::ChannelPost(post) => post.text(),
        }
    }
}

impl MessageText for Message {
    fn text<'a>(&'a self) -> Option<String> {
        self.kind.text()
    }
}

impl MessageText for MessageKind {
    fn text<'a>(&'a self) -> Option<String> {
        match self {
            MessageKind::Text { data, .. } => Some(data.to_owned()),
            MessageKind::Audio { data } => data.title.to_owned(),
            MessageKind::Document { data, caption } => {
                caption.clone().or_else(|| data.file_name.clone())
            }
            MessageKind::Photo { caption, .. } => caption.to_owned(),
            MessageKind::Sticker { .. } => None,
            MessageKind::Video { caption, .. } => caption.to_owned(),
            MessageKind::Voice { .. } => None,
            MessageKind::VideoNote { .. } => None,
            MessageKind::Contact { data } => Some(data.first_name.to_owned()),
            MessageKind::Location { .. } => None,
            MessageKind::Venue { data } => Some(data.title.to_owned()),
            MessageKind::NewChatMembers { .. } => None,
            MessageKind::LeftChatMember { .. } => None,
            MessageKind::NewChatTitle { data } => Some(data.to_owned()),
            MessageKind::NewChatPhoto { .. } => None,
            MessageKind::DeleteChatPhoto => None,
            MessageKind::GroupChatCreated => None,
            MessageKind::SupergroupChatCreated => None,
            MessageKind::ChannelChatCreated => None,
            MessageKind::MigrateToChatId { .. } => None,
            MessageKind::MigrateFromChatId { .. } => None,
            MessageKind::PinnedMessage { data } => data.text(),
            MessageKind::Unknown { .. } => None,
        }
    }
}

impl MessageText for ChannelPost {
    fn text<'a>(&'a self) -> Option<String> {
        self.kind.text()
    }
}

/// A trait to obtain `GetFile` requests from a message.
pub trait MessageGetFiles {
    /// Obtain files from a message if available.
    fn files<'a>(&'a self) -> Option<Vec<GetFile>>;
}

impl MessageGetFiles for MessageOrChannelPost {
    fn files<'a>(&'a self) -> Option<Vec<GetFile>> {
        match self {
            MessageOrChannelPost::Message(msg) => msg.files(),
            MessageOrChannelPost::ChannelPost(post) => post.files(),
        }
    }
}

impl MessageGetFiles for Message {
    fn files<'a>(&'a self) -> Option<Vec<GetFile>> {
        self.kind.files()
    }
}

impl MessageGetFiles for MessageKind {
    fn files<'a>(&'a self) -> Option<Vec<GetFile>> {
        match self {
            MessageKind::Text { .. } => None,
            MessageKind::Audio { data } => Some(vec![data.get_file()]),
            MessageKind::Document { data, .. } => Some(vec![data.get_file()]),
            MessageKind::Photo { data, .. } => Some(data
                .into_iter()
                .map(|f| f.get_file())
                .collect()
            ),
            MessageKind::Sticker { data } => Some(vec![data.get_file()]),
            MessageKind::Video { data, .. } => Some(vec![data.get_file()]),
            MessageKind::Voice { data } => Some(vec![data.get_file()]),
            MessageKind::VideoNote { data } => Some(vec![data.get_file()]),
            MessageKind::Contact { .. } => None,
            MessageKind::Location { .. } => None,
            MessageKind::Venue { .. } => None,
            MessageKind::NewChatMembers { .. } => None,
            MessageKind::LeftChatMember { .. } => None,
            MessageKind::NewChatTitle { .. } => None,
            MessageKind::NewChatPhoto { data } => Some(data
                .into_iter()
                .map(|f| f.get_file())
                .collect()
            ),
            MessageKind::DeleteChatPhoto => None,
            MessageKind::GroupChatCreated => None,
            MessageKind::SupergroupChatCreated => None,
            MessageKind::ChannelChatCreated => None,
            MessageKind::MigrateToChatId { .. } => None,
            MessageKind::MigrateFromChatId { .. } => None,
            MessageKind::PinnedMessage { .. } => None,
            MessageKind::Unknown { .. } => None,
        }
    }
}

impl MessageGetFiles for ChannelPost {
    fn files<'a>(&'a self) -> Option<Vec<GetFile>> {
        self.kind.files()
    }
}
