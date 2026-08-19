use derive_more::From;
use iroh::endpoint::ConnectError;
use crate::tunnel_protocol::Error as TunnelProtocolError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Custom(String),

    #[from]
    BindError(iroh::endpoint::BindError),

    #[from]
    ConnectError(ConnectError),

    #[from]
    IoError(std::io::Error),

    #[from]
    TunnelProtocolError(TunnelProtocolError),

    #[from]
    ConnectionError(iroh::endpoint::ConnectionError),
}

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
