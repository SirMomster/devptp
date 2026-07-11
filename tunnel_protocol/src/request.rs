use crate::{error::Result, Error};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};

#[derive(Debug, Clone, Copy)]
pub enum Request {
    Open { port: u16 },
}

impl Request {
    pub async fn write_to<W>(&self, writer: &mut W) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        match self {
            Request::Open { port } => {
                writer
                    .write_all(format!("OPEN {port}\n").as_bytes())
                    .await?;
            }
        }
        writer.flush().await?;
        Ok(())
    }

    pub async fn read_from<R>(reader: &mut BufReader<R>) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let line = line.trim();
        let port = line.strip_prefix("OPEN ").ok_or(Error::PortParseError {
            port: "UNKNOWN".to_string(),
        })?;

        let port = port.parse::<u16>().map_err(|_| Error::PortParseError {
            port: port.to_string(),
        })?;

        Ok(Request::Open { port })
    }
}
