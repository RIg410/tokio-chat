use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use anyhow::Error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use x25519_dalek;
use x25519_dalek::{PublicKey, StaticSecret};

pub type PublicKeyBuf = [u8; 32];
pub type NonceBuf = [u8; 12];

#[derive(Serialize, Deserialize, Debug)]
pub struct ContentBox<T> {
    phantom: PhantomData<T>,
    content: Vec<u8>,
    sender: PublicKeyBuf,
    nonce: NonceBuf,
    respondents: HashMap<PublicKeyBuf, (NonceBuf, Vec<u8>)>,
}

impl<T: Serialize + DeserializeOwned> ContentBox<T> {
    pub fn encode(
        content: T,
        sender: StaticSecret,
        respondents: Vec<PublicKey>,
    ) -> Result<Self, Error> {
        let content = bincode::serialize(&content)?;
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce_buf = rand_buf();
        let cipher_context = cipher
            .encrypt(&Nonce::from_slice(&nonce_buf), content.as_ref())
            .map_err(|err| anyhow::anyhow!(err))?;

        let sender_public = PublicKey::from(&sender);

        let respondents = respondents
            .iter()
            .map(|r| {
                let shared_secret = sender.diffie_hellman(r);
                let cipher = Aes256Gcm::new_from_slice(shared_secret.as_bytes())?;
                let nonce_buf = rand_buf();
                let cipher_context = cipher
                    .encrypt(&&Nonce::from_slice(&nonce_buf), key.as_slice())
                    .map_err(|err| anyhow::anyhow!(err))?;
                Ok((r.clone().to_bytes(), (nonce_buf, cipher_context)))
            })
            .collect::<Result<_, Error>>()?;

        Ok(Self {
            phantom: PhantomData,
            content: cipher_context,
            sender: sender_public.to_bytes(),
            nonce: nonce_buf,
            respondents,
        })
    }

    pub fn content(&self, recipient: &StaticSecret) -> Result<T, Error> {
        let sender_public = PublicKey::from(recipient);
        let (nonce, encoded_secret) = self
            .respondents
            .get(&sender_public.to_bytes())
            .ok_or_else(|| anyhow::anyhow!("No such recipient"))?;

        let shared_secret = recipient.diffie_hellman(&PublicKey::from(self.sender));
        let cipher = Aes256Gcm::new_from_slice(shared_secret.as_bytes())?;
        let key = cipher
            .decrypt(&Nonce::from_slice(nonce), encoded_secret.as_ref())
            .map_err(|err| anyhow::anyhow!(err))?;

        let cipher = Aes256Gcm::new_from_slice(key.as_slice())?;
        cipher
            .decrypt(&Nonce::from_slice(&self.nonce), self.content.as_ref())
            .map_err(|err| anyhow::anyhow!(err))
            .and_then(|content| bincode::deserialize(&content).map_err(|err| anyhow::anyhow!(err)))
    }
}

pub fn rand_buf<const SIZE: usize>() -> [u8; SIZE] {
    let mut nonce = [0u8; SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[test]
fn test() {
    let sender = StaticSecret::from(rand_buf::<32>());
    let recipient = StaticSecret::from(rand_buf::<32>());
    let recipient2 = StaticSecret::from(rand_buf::<32>());

    let content = ContentBox::encode(
        "Hello, world!".to_string(),
        sender,
        vec![PublicKey::from(&recipient), PublicKey::from(&recipient2)],
    )
    .unwrap();

    assert_eq!(content.content(&recipient).unwrap(), "Hello, world!");
    assert_eq!(content.content(&recipient2).unwrap(), "Hello, world!");
}
