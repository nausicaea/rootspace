use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub(crate) enum RpcMessage {
    Hello(String, SocketAddr),
    Exit,
}
