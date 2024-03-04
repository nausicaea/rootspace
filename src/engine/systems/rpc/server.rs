use crate::engine::systems::rpc::message::RpcMessage;
use crate::engine::systems::rpc::service::RpcService;
use std::net::SocketAddr;
use tarpc::context::Context;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct RpcServer {
    mpsc_tx: Sender<RpcMessage>,
    socket_address: SocketAddr,
}

impl RpcServer {
    pub(crate) fn new(mpsc_tx: Sender<RpcMessage>, socket_address: SocketAddr) -> Self {
        RpcServer {
            mpsc_tx,
            socket_address,
        }
    }
}

impl RpcService for RpcServer {
    async fn hello(self, _context: Context, name: String) -> String {
        let output = format!("Hello, {}@{}", &name, self.socket_address);
        self.mpsc_tx
            .send(RpcMessage::Hello(name, self.socket_address))
            .await
            .unwrap();
        output
    }
}
