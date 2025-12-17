use futures::channel::{mpsc::SendError, oneshot::Canceled};

use crate::resources::statistics::Statistics;
use crate::systems::rpc::graphics_info::{GraphicsInfo, GraphicsInfoCategory};

#[tarpc::service]
pub trait RpcService {
    /// Requests the engine to exit
    async fn exit() -> Result<(), Error>;
    /// Requests information about the graphics subsystem
    async fn graphics_info(category: GraphicsInfoCategory) -> Result<GraphicsInfo, Error>;
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

impl From<SendError> for Error {
    fn from(value: SendError) -> Self {
        Error::MpscSend(format!("{}", value))
    }
}

impl From<Canceled> for Error {
    fn from(value: Canceled) -> Self {
        Error::OneshotRecv(format!("{}", value))
    }
}
