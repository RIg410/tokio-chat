use crate::crypto::content_box::ContentBox;
use crate::messaging::UserId;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use snow::Builder;
use std::collections::HashMap;
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    content: ContentBox<Content>,
}

impl Message {
    pub fn new(
        content: Content,
        sender: StaticSecret,
        respondents: Vec<PublicKey>,
    ) -> Result<Message, Error> {
        let content = ContentBox::encode(content, sender, respondents)?;
        Ok(Message { content })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    parts: Vec<MessagePart>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessagePart {
    Text(String),
}
