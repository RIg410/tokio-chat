use crate::messaging::{ChatId, UserId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub enum User {
    GetInfo(UserId, Vec<UserParams>),
    SetParams(UserId, HashMap<UserParams, String>),
    Remove(ChatId),
    SendInvite(ChatId, UserId),
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum UserParams {
    Nickname,
    Key,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct UserInfo {
    info: HashMap<UserParams, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserData {
    Info(UserInfo),
}
