use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Custom(String),

    #[from]
    IrohBindError(iroh::endpoint::BindError),
    #[from]
    IrohConnectError(iroh::endpoint::ConnectError),
    #[from]
    IrohClosedStreamError(iroh::endpoint::ClosedStream),
    #[from]
    IrohWriteError(iroh::endpoint::WriteError),
    #[from]
    IrohConnectionError(iroh::endpoint::ConnectionError),
    #[from]
    IrohReadToEndError(iroh::endpoint::ReadToEndError),

    #[from]
    IoError(std::io::Error),
}

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(fmt, "{self:?}")
    }
}
