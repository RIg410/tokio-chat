use crate::messaging::chat::ChatData;
use crate::messaging::message::Message;
use crate::messaging::user::UserData;
use crate::messaging::{ChatId, RequestId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Notification {
    DirectMessage(UserId, Message),
    ChatMessage(ChatId, Message),
    DirectInvite(UserId),
    ChatInvite(ChatId),
    ChatUpdated(ChatId),
    Response(RequestId, RequestedData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestedData {
    Chat(ChatData),
    User(UserData),
}
