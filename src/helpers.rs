use serde::{de::DeserializeOwned, Serialize};
use std::{io, net::TcpListener};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

pub async fn send_json<W, T>(writer: &mut W, message: &T) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let mut bytes = serde_json::to_vec(message).map_err(io::Error::other)?;

    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    writer.flush().await
}

pub async fn receive_json<R, T>(reader: &mut R) -> io::Result<Option<T>>
where
    R: AsyncBufRead + Unpin,
    T: DeserializeOwned,
{
    let mut line = String::new();

    if reader.read_line(&mut line).await? == 0 {
        return Ok(None); // connection closed
    }

    serde_json::from_str(&line)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn port_is_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub fn get_first_available_port(start_port: u16) -> Option<u16> {
    let end_port = u16::MAX - 1;
    (start_port..=end_port).find(|port| port_is_available(*port))
}
