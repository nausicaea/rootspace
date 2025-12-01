use std::net::SocketAddr;

use super::service::Error;
use crate::systems::rpc::graphics_info::{GraphicsInfo, GraphicsInfoCategory};
use crate::{
    resources::statistics::Statistics,
    systems::rpc::{message::RpcMessage, service::RpcService},
};
use tarpc::context::Context;
use tokio::sync::mpsc;
use tracing::trace;

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
    #[tracing::instrument(skip_all, fields(client = self.socket_address.to_string()))]
    async fn exit(self, _: Context) -> Result<(), Error> {
        trace!("RpcService::exit");
        self.mpsc_tx.send(RpcMessage::Exit).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, _context), fields(client = self.socket_address.to_string()))]
    async fn graphics_info(self, _context: Context, category: GraphicsInfoCategory) -> Result<GraphicsInfo, Error> {
        trace!("RpcService::graphics_info");
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.mpsc_tx.send(RpcMessage::GraphicsInfo { tx, category }).await?;
        let info = rx.await?;
        Ok(info)
    }

    #[tracing::instrument(skip_all, fields(client = self.socket_address.to_string()))]
    async fn perf(self, _: Context) -> Result<Statistics, Error> {
        trace!("RpcService::perf");
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.mpsc_tx.send(RpcMessage::StatsRequest(tx)).await?;
        let stats = rx.await?;
        Ok(stats)
    }

    #[tracing::instrument(skip_all, fields(client = self.socket_address.to_string()))]
    async fn load_scene(self, _: Context, group: String, name: String) -> Result<(), Error> {
        trace!("RpcService::load_scene");
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.mpsc_tx.send(RpcMessage::LoadScene { tx, group, name }).await?;
        rx.await??;
        Ok(())
    }
}
