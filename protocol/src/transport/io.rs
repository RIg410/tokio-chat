use anyhow::{anyhow, ensure, Error};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde::Serialize;
use snow::TransportState;
use std::cmp::min;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::{spawn_blocking, spawn_local};

pub const TAG_SIZE: usize = 16;
pub const SOURCE_CHUNK_SIZE: usize = 65535 - TAG_SIZE;
pub const ENC_CHUNK_SIZE: usize = 65535;

pub struct EncryptedStream {
    inner: Option<Inner>,
}

struct Inner {
    tcp: TcpStream,
    state: TransportState,
}

// todo optimize memory usage
impl EncryptedStream {
    pub fn new(tcp: TcpStream, state: TransportState) -> Self {
        Self {
            inner: Some(Inner { tcp, state }),
        }
    }

    pub async fn send<T: Serialize + Send>(&mut self, val: T) -> Result<(), Error> {
        let mut inner = self.inner.take().ok_or_else(|| anyhow!("Invalid state"))?;
        let (inner, buffer) = spawn_blocking(move || {
            let mut buf = bincode::serialize(&val)?;
            let mut encoded_chunks = Vec::with_capacity(buf.len() / SOURCE_CHUNK_SIZE + 1);
            let mut buf = buf.as_slice();

            while !buf.is_empty() {
                let (chunk, rest) = buf.split_at(SOURCE_CHUNK_SIZE);
                buf = rest;
                let mut encoded_chunk = vec![0; chunk.len() + TAG_SIZE];
                let len = inner.state.write_message(chunk, &mut encoded_chunk)?;
                encoded_chunks.truncate(len);
                encoded_chunks.push(encoded_chunk);
            }
            (inner, Ok(encoded_chunks))
        })
        .await??;

        let len: usize = buffer.iter().map(|buf| buf.len()).sum();
        self.tcp.write_u32(len as u32).await?;
        for buf in buffer {
            self.tcp.write_all(&buf).await?;
        }
        Ok(())
    }

    // pub async fn read<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
    //     let len = self.tcp.read_u32().await?;
    //     let mut buf = vec![0; len as usize];
    //     let buf_len = self.tcp.read_exact(&mut buf).await?;
    //     ensure!(buf.len() == buf_len, "invalid message length");
    //     Ok(spawn_blocking(|| {
    //         let mut source = Vec::with_capacity(buf.len());
    //         let mut buf = buf.as_slice();
    //         while buf.len() > 0 {
    //             let chunk_len = min(buf.len(), ENC_CHUNK_SIZE);
    //             let (chunk, rest) = buf.split_at(chunk_len);
    //             buf = rest;
    //             let mut decoded_chunk = vec![0; chunk.len()];
    //             let len = self.state.read_message(chunk, &mut decoded_chunk)?;
    //             source.extend_from_slice(&decoded_chunk[..len]);
    //         }
    //     })
    //     .await??)
    // }
}
