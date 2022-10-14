use crate::messaging::message::Message;
use crate::messaging::UserId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Direct {
    SendInvite(UserId),
    AcceptInvite(UserId),
    ChatMessage(UserId, Message),
}
