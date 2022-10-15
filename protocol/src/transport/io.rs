use anyhow::{anyhow, ensure, Error};
use serde::de::DeserializeOwned;
use serde::Serialize;
use snow::TransportState;
use std::cmp::min;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::spawn_blocking;

pub const TAG_SIZE: usize = 16;
pub const SOURCE_CHUNK_SIZE: usize = 65535 - TAG_SIZE;
pub const ENC_CHUNK_SIZE: usize = 65535;

pub struct EncryptedStream {
    tcp: TcpStream,
    state: Option<TransportState>,
}

impl EncryptedStream {
    pub fn new(tcp: TcpStream, state: TransportState) -> Self {
        Self {
            tcp,
            state: Some(state),
        }
    }

    // todo optimize memory usage
    pub async fn send<T: Serialize + Send + 'static>(&mut self, val: T) -> Result<(), Error> {
        let mut state = self.state.take().ok_or_else(|| anyhow!("Invalid state"))?;
        let res = spawn_blocking(move || {
            let buf = match bincode::serialize(&val) {
                Ok(buf) => buf,
                Err(err) => return (state, Err(Error::new(err))),
            };

            let mut encoded_chunks = Vec::with_capacity(buf.len() / SOURCE_CHUNK_SIZE + 1);
            let mut buf = buf.as_slice();

            while !buf.is_empty() {
                let chunk_len = min(buf.len(), SOURCE_CHUNK_SIZE);
                let (chunk, rest) = buf.split_at(chunk_len);
                buf = rest;
                let mut encoded_chunk = vec![0; chunk.len() + TAG_SIZE];

                let len = match state.write_message(chunk, &mut encoded_chunk) {
                    Ok(len) => len,
                    Err(err) => return (state, Err(Error::new(err))),
                };
                encoded_chunks.truncate(len);
                encoded_chunks.push(encoded_chunk);
            }
            (state, Ok(encoded_chunks))
        })
        .await;
        let buffer = match res {
            Ok((state, buf)) => {
                self.state = Some(state);
                buf?
            }
            Err(err) => {
                panic!("Error: {}. Panic on spawning local task.", err);
            }
        };

        let len: usize = buffer.iter().map(|buf| buf.len()).sum();
        self.tcp.write_u32(len as u32).await?;
        for buf in buffer {
            self.tcp.write_all(&buf).await?;
        }
        Ok(())
    }

    // todo optimize memory usage
    pub async fn read<T: DeserializeOwned + Send + 'static>(&mut self) -> Result<T, Error> {
        let len = self.tcp.read_u32().await?;
        let mut buf = vec![0; len as usize];
        let buf_len = self.tcp.read_exact(&mut buf).await?;
        ensure!(buf.len() == buf_len, "invalid message length");

        let mut state = self.state.take().ok_or_else(|| anyhow!("Invalid state"))?;
        let res = spawn_blocking(move || {
            let mut source = Vec::with_capacity(buf.len());
            let mut buf = buf.as_slice();
            while buf.len() > 0 {
                let chunk_len = min(buf.len(), ENC_CHUNK_SIZE);
                let (chunk, rest) = buf.split_at(chunk_len);
                buf = rest;
                let mut decoded_chunk = vec![0; chunk.len()];
                let len = match state.read_message(chunk, &mut decoded_chunk) {
                    Ok(len) => len,
                    Err(err) => return (state, Err(Error::new(err))),
                };
                source.extend_from_slice(&decoded_chunk[..len]);
            }
            match bincode::deserialize(&source) {
                Ok(val) => (state, Ok(val)),
                Err(err) => (state, Err(Error::new(err))),
            }
        })
        .await;
        match res {
            Ok((state, val)) => {
                self.state = Some(state);
                val
            }
            Err(err) => {
                panic!("Error: {}. Panic on spawning local task.", err);
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::crypto::content_box::rand_buf;
    use crate::transport::handshake::tests::pair_socket;
    use crate::transport::handshake::{client_handshake, server_handshake};
    use crate::transport::io::EncryptedStream;
    use rand_core::{OsRng, RngCore};
    use tokio::join;
    use x25519_dalek::{PublicKey, StaticSecret};

    #[tokio::test]
    async fn test_read_write_short_message() {
        let server_key = StaticSecret::from(rand_buf::<32>());
        let (mut cs, mut ss) = pair_socket(9912).await;
        let server = server_handshake(&mut ss, server_key.clone());
        let client = client_handshake(&mut cs, PublicKey::from(&server_key));
        let (server, client) = join!(server, client);
        let (server, client) = (server.unwrap(), client.unwrap());
        let mut server = EncryptedStream::new(ss, server);
        let mut client = EncryptedStream::new(cs, client);

        client.send("This is a short test message").await.unwrap();
        let msg: String = server.read().await.unwrap();
        assert_eq!(msg, "This is a short test message");

        server.send("This is a short test message").await.unwrap();
        let msg: String = client.read().await.unwrap();
        assert_eq!(msg, "This is a short test message");
    }

    #[tokio::test]
    async fn test_read_write_with_large_message() {
        let server_key = StaticSecret::from(rand_buf::<32>());
        let (mut cs, mut ss) = pair_socket(9913).await;
        let server = server_handshake(&mut ss, server_key.clone());
        let client = client_handshake(&mut cs, PublicKey::from(&server_key));
        let (server, client) = join!(server, client);
        let (server, client) = (server.unwrap(), client.unwrap());
        let mut server = EncryptedStream::new(ss, server);
        let mut client = EncryptedStream::new(cs, client);

        let mut msg = vec![0; 1024 * 1024];
        OsRng.fill_bytes(&mut msg);
        client.send(msg.clone()).await.unwrap();
        let msg2: Vec<u8> = server.read().await.unwrap();
        assert_eq!(msg, msg2);

        let mut msg = vec![0; 1024 * 1024];
        OsRng.fill_bytes(&mut msg);
        server.send(msg.clone()).await.unwrap();
        let msg2: Vec<u8> = client.read().await.unwrap();
        assert_eq!(msg, msg2);
    }
}
