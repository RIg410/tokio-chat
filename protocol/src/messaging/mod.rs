use crate::messaging::chat::Chat;
use crate::messaging::direct::Direct;
use crate::messaging::transport::Transport;
use crate::messaging::user::User;
use serde::{Deserialize, Serialize};

pub mod chat;
pub mod direct;
pub mod message;
pub mod transport;
pub mod user;

pub type UserId = u128;
pub type ChatId = u128;
pub type RequestId = u128;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    User(User),
    Transport(Transport),
    Chat(Chat),
    Direct(Direct),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Ok(RequestId),
    Err(String),
}
