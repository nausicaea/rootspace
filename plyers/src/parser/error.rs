#[derive(Debug, thiserror::Error)]
#[error("received byte {received:#x}, but expected byte {expected:#x} in engram {engram:?}")]
pub struct EngramError {
    pub received: u8,
    pub expected: u8,
    pub engram: &'static [u8],
}

impl EngramError {
    pub fn new(received: u8, expected: u8, engram: &'static [u8]) -> Self {
        EngramError { received, expected, engram }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("received byte {received:#x}, but expected byte {expected:#x}")]
pub struct LookaheadError {
    received: u8,
    expected: u8,
}

impl LookaheadError {
    pub fn new(received: u8, expected: u8) -> Self {
        LookaheadError { received, expected }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("received byte {received:#x}, but expected byte {expected:#x}")]
pub struct TokenError {
    received: u8,
    expected: u8,
}

impl TokenError {
    pub fn new(received: u8, expected: u8) -> Self {
        TokenError { received, expected }
    }
}

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
