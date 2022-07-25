#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("unable to read the stream position")]
    PositionRead(#[source] std::io::Error),
    #[error("unable to seek within the stream")]
    PositionWrite(#[source] std::io::Error),
    #[error("unable to read a byte from the stream")]
    Read(#[source] std::io::Error),
    #[error("byte stream ended unexpectedly")]
    EndOfStream,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{source} in file {}", .path.display())]
pub struct FileWrapper<E: std::error::Error> {
    pub source: E,
    pub path: std::path::PathBuf,
}

impl<E: std::error::Error> FileWrapper<E> {
    pub fn new(source: E, path: std::path::PathBuf) -> Self {
        FileWrapper { source, path }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{source} with surrounding bytes {context:?}")]
pub struct SurroundingBytesWrapper<E: std::error::Error> {
    pub source: E,
    pub context: [u8; 32],
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{source} at address {address:#x} (dec. {address})")]
pub struct AddressWrapper<E: std::error::Error> {
    pub source: E,
    pub address: u64,
}

impl<E: std::error::Error> AddressWrapper<E> {
    pub fn new(source: E, address: u64) -> Self {
        AddressWrapper { source, address }
    }
}
