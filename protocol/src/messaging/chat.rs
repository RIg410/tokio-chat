use crate::messaging::message::Message;
use crate::messaging::{ChatId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Chat {
    Create,
    List,
    Info(ChatId),
    Remove(ChatId),
    SendInvite(ChatId, UserId),
    AcceptInvite(ChatId),
    ChatMessage(ChatId, Message),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChatData {
    Info(ChatInfo),
    List(ChatList),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatList {
    chats: Vec<ChatId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatInfo {
    id: ChatId,
    users: Vec<UserId>,
}
