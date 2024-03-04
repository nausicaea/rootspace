use std::net::SocketAddr;
use tokio::sync::oneshot::Sender;
use crate::engine::resources::statistics::Statistics;

#[derive(Debug)]
pub(crate) enum RpcMessage {
    Hello(String, SocketAddr),
    StatsRequest(Sender<Statistics>),
    Exit,
}
