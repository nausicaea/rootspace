use tokio::sync::{mpsc::error::SendError, oneshot::error::RecvError};

use super::message::RpcMessage;
use crate::resources::statistics::Statistics;

#[tarpc::service]
pub trait RpcService {
    /// Returns a greeting for name.
    async fn hello(name: String) -> Result<String, Error>;
    /// Requests the engine to exit
    async fn exit() -> Result<(), Error>;
    /// Requests performance statistics
    async fn perf() -> Result<Statistics, Error>;
    /// Requests the engine to load a scene
    async fn load_scene(group: String, name: String) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum Error {
    #[error("When sending data from the RPC server to the engine: {}", .0)]
    MpscSend(String),
    #[error("When the RPC server expected data from the engine: {}", .0)]
    OneshotRecv(String),
    #[error("{}", .0)]
    Other(String),
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Error::Other(format!("{:?}", value))
    }
}

impl From<SendError<RpcMessage>> for Error {
    fn from(value: SendError<RpcMessage>) -> Self {
        Error::MpscSend(format!("{}", value))
    }
}

impl From<RecvError> for Error {
    fn from(value: RecvError) -> Self {
        Error::OneshotRecv(format!("{}", value))
    }
}
