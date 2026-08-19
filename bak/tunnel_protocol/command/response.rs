use super::super::error::{Error, Result};

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Ok,
    Error { message: String },
    Port(u16),
}

impl Response {
    pub async fn write_to<W>(&self, writer: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        match self {
            Self::Ok => {
                writer.write_all(b"OK\n").await?;
            }

            Self::Error { message } => {
                // Keep the response line-based by preventing embedded newlines.
                let message = message.replace(['\r', '\n'], " ");

                writer.write_all(b"ERR ").await?;
                writer.write_all(message.as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }

            Self::Port(port) => {
                writer
                    .write_all(format!("PORT {port}\n").as_bytes())
                    .await?;
            }
        }

        writer.flush().await?;

        Ok(())
    }

    pub async fn read_from<R>(reader: &mut R) -> Result<Self>
    where
        R: AsyncBufRead + Unpin,
    {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Err(Error::Custom(
                "connection closed before a tunnel response was received".into(),
            ));
        }

        let line = line.trim_end_matches(['\r', '\n']);
        if line == "OK" {
            return Ok(Self::Ok);
        }

        if let Some(message) = line.strip_prefix("ERR ") {
            return Ok(Self::Error {
                message: message.to_owned(),
            });
        }

        if line == "ERR" {
            return Ok(Self::Error {
                message: String::new(),
            });
        }

        if let Some(port_str) = line.strip_prefix("PORT ") {
            let port = port_str.parse::<u16>().map_err(|_| Error::PortParseError {
                port: port_str.to_string(),
            })?;
            return Ok(Self::Port(port));
        }

        Err(Error::Custom(format!("invalid tunnel response: {line:?}")))
    }
}
