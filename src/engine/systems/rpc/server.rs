use crate::engine::resources::statistics::Statistics;
use crate::engine::systems::rpc::message::RpcMessage;
use crate::engine::systems::rpc::service::RpcService;
use log::trace;
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
        trace!("RpcService::hello");
        let output = format!("Hello, {}@{}", &name, self.socket_address);
        self.mpsc_tx
            .send(RpcMessage::Hello(name, self.socket_address))
            .await
            .unwrap();
        output
    }

    async fn exit(self, _: Context) {
        trace!("RpcService::exit");
        self.mpsc_tx.send(RpcMessage::Exit).await.unwrap();
    }

    async fn perf(self, _: Context) -> Statistics {
        trace!("RpcService::perf");
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.mpsc_tx.send(RpcMessage::StatsRequest(tx)).await.unwrap();
        let stats = rx.await.unwrap();
        stats
    }
}
