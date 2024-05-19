use std::net::SocketAddr;

use tarpc::context::Context;
use tokio::sync::mpsc;

use super::service::Error;
use crate::{
    resources::statistics::Statistics,
    systems::rpc::{message::RpcMessage, service::RpcService},
};

#[derive(Debug, Clone)]
pub struct RpcServer {
    mpsc_tx: mpsc::Sender<RpcMessage>,
    socket_address: SocketAddr,
}

impl RpcServer {
    pub fn new(mpsc_tx: mpsc::Sender<RpcMessage>, socket_address: SocketAddr) -> Self {
        RpcServer {
            mpsc_tx,
            socket_address,
        }
    }
}

impl RpcService for RpcServer {
    async fn hello(self, _context: Context, name: String) -> Result<String, Error> {
        tracing::trace!("RpcService::hello");
        Ok(format!("Hello, {}@{}", &name, self.socket_address))
    }

    async fn exit(self, _: Context) -> Result<(), Error> {
        tracing::trace!("RpcService::exit");
        self.mpsc_tx.send(RpcMessage::Exit).await?;
        Ok(())
    }

    async fn perf(self, _: Context) -> Result<Statistics, Error> {
        tracing::trace!("RpcService::perf");
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.mpsc_tx.send(RpcMessage::StatsRequest(tx)).await?;
        let stats = rx.await?;
        Ok(stats)
    }

    async fn load_scene(self, _: Context, group: String, name: String) -> Result<(), Error> {
        tracing::trace!("RpcService::load_scene");
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.mpsc_tx.send(RpcMessage::LoadScene { tx, group, name }).await?;
        rx.await??;
        Ok(())
    }
}
