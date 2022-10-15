use anyhow::Error;
use snow::TransportState;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use x25519_dalek::{PublicKey, StaticSecret};

static PATTERN: &'static str = "Noise_NN_25519_ChaChaPoly_BLAKE2s";

pub async fn client_handshake(
    soc: &mut TcpStream,
    server: PublicKey,
) -> Result<TransportState, Error> {
    let mut initiator = snow::Builder::new(PATTERN.parse()?)
        .remote_public_key(server.as_bytes())
        .build_initiator()?;
    let mut buf = [0u8; 1024];
    let len = initiator.write_message(&[], &mut buf)?;
    soc.write_u32(len as u32).await?;
    soc.write_all(&buf[..len]).await?;
    let len = soc.read_u32().await? as usize;
    soc.read_exact(&mut buf[..len]).await?;
    let mut decoded = [0u8; 1024];
    initiator.read_message(&buf[..len], &mut decoded)?;
    Ok(initiator.into_transport_mode()?)
}

pub async fn server_handshake(
    soc: &mut TcpStream,
    server: StaticSecret,
) -> Result<TransportState, Error> {
    let mut responder = snow::Builder::new(PATTERN.parse()?)
        .local_private_key(&server.to_bytes())
        .build_responder()?;
    let mut encoded = [0u8; 1024];
    let mut decoded = [0u8; 1024];
    let len = soc.read_u32().await?;
    soc.read_exact(&mut encoded[..len as usize]).await?;
    responder.read_message(&encoded[..len as usize], &mut decoded)?;
    let len = responder.write_message(&[], &mut encoded)?;
    soc.write_u32(len as u32).await?;
    soc.write_all(&encoded[..len]).await?;
    Ok(responder.into_transport_mode()?)
}

#[cfg(test)]
pub mod tests {
    use crate::crypto::content_box::rand_buf;
    use tokio::join;
    use tokio::net::{TcpListener, TcpStream};
    use x25519_dalek::{PublicKey, StaticSecret};

    pub async fn pair_socket(port: u16) -> (TcpStream, TcpStream) {
        let server_soc = TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
        let server_soc = server_soc.accept();
        let client_soc = TcpStream::connect(format!("0.0.0.0:{port}"));
        let (server, client) = join!(server_soc, client_soc);
        (server.unwrap().0, client.unwrap())
    }

    #[tokio::test]
    async fn test_handshake() {
        let server_key = StaticSecret::from(rand_buf::<32>());
        let (mut server, mut client) = pair_socket(9911).await;

        let server = super::server_handshake(&mut server, server_key.clone());
        let client = super::client_handshake(&mut client, PublicKey::from(&server_key));
        let (server, client) = join!(server, client);
        let (_server, _client) = (server.unwrap(), client.unwrap());
    }
}
