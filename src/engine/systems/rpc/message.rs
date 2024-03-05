use crate::engine::resources::statistics::Statistics;
use std::net::SocketAddr;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub(crate) enum RpcMessage {
    Hello(String, SocketAddr),
    StatsRequest(Sender<Statistics>),
    Exit,
}
