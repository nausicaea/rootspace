pub mod graphics_info;
mod message;
mod server;
pub mod service;

use std::{future::ready, time::Duration};

use async_trait::async_trait;
use futures::StreamExt;
use message::RpcMessage;
use tarpc::server::{BaseChannel, Channel, incoming::Incoming};
use tokio::sync::oneshot;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::error;

use crate::{
    assets::scene::Scene,
    events::engine_event::EngineEvent,
    resources::{rpc_settings::RpcSettings, statistics::Statistics},
    systems::rpc::{server::RpcServer, service::RpcService},
};
use assam::AssetDatabase;
use ecs::{
    event_queue::{EventQueue, receiver_id::ReceiverId},
    resources::Resources,
    system::System,
    with_resources::WithResources,
};
use graphics_info::GraphicsInfo;
use graphics_info::GraphicsInfoCategory;
use griffon::Graphics;

#[derive(Debug)]
pub struct Rpc {
    rpc_listener: JoinHandle<()>,
    mpsc_rx: mpsc::Receiver<RpcMessage>,
    receiver: ReceiverId<EngineEvent>,
}

impl Rpc {
    #[tracing::instrument(skip_all)]
    async fn perf(&self, res: &Resources, tx: oneshot::Sender<Statistics>) {
        let stats = res.read::<Statistics>().clone();
        if tx.send(stats).is_err() {
            error!("unable to send statistics data to the RPC server");
        }
    }

    #[tracing::instrument(skip_all)]
    async fn graphics_info(&self, res: &Resources, tx: oneshot::Sender<GraphicsInfo>, category: GraphicsInfoCategory) {
        let gfx = res.read::<Graphics>();
        let response = match category {
            GraphicsInfoCategory::InstanceReport => {
                GraphicsInfo::InstanceReport(Box::new(gfx.gen_instance_report().map(Into::into)))
            }
            GraphicsInfoCategory::SurfaceCapabilities => {
                GraphicsInfo::SurfaceCapabilities(gfx.gen_surface_capabilities().into())
            }
            GraphicsInfoCategory::AdapterFeatures => GraphicsInfo::AdapterFeatures(gfx.gen_adapter_features()),
            GraphicsInfoCategory::AdapterLimits => GraphicsInfo::AdapterLimits(gfx.gen_adapter_limits()),
            GraphicsInfoCategory::AdapterDownlevelCapabilities => {
                GraphicsInfo::AdapterDownlevelCapabilities(gfx.gen_adapter_downlevel_capabilities())
            }
            GraphicsInfoCategory::AdapterInfo => GraphicsInfo::AdapterInfo(gfx.gen_adapter_info()),
            GraphicsInfoCategory::DeviceAllocatorReport => {
                GraphicsInfo::DeviceAllocatorReport(gfx.gen_device_allocator_report().map(Into::into))
            }
        };
        if tx.send(response).is_err() {
            error!("unable to send the graphics subsystem info response to the RPC server");
        }
    }

    #[tracing::instrument(skip_all)]
    async fn load_scene(&self, res: &Resources, tx: oneshot::Sender<anyhow::Result<()>>, group: &str, name: &str) {
        let r = res
            .read::<AssetDatabase>()
            .load_asset::<Scene, _>(res, group, name)
            .await;
        if tx.send(r).is_err() {
            error!("unable to send the result of asset loading to the RPC server");
        }
    }

    #[tracing::instrument(skip_all)]
    async fn exit(&self, res: &Resources) {
        res.write::<EventQueue<EngineEvent>>().send(EngineEvent::Exit)
    }
}

#[async_trait]
impl System for Rpc {
    #[tracing::instrument(skip_all)]
    async fn run(&mut self, res: &Resources, _t: Duration, _dt: Duration) {
        let events = res.write::<EventQueue<EngineEvent>>().receive(&self.receiver);
        for event in events {
            #[allow(irrefutable_let_patterns)]
            if let EngineEvent::Exit = event {
                tracing::trace!("Stopping RPC listener");
                self.rpc_listener.abort();
            }
        }

        while let Ok(msg) = self.mpsc_rx.try_recv() {
            match msg {
                RpcMessage::StatsRequest(tx) => self.perf(res, tx).await,
                RpcMessage::GraphicsInfo { tx, category } => self.graphics_info(res, tx, category).await,
                RpcMessage::LoadScene {
                    tx,
                    ref group,
                    ref name,
                } => self.load_scene(res, tx, group, name).await,
                RpcMessage::Exit => self.exit(res).await,
            }
        }
    }
}

impl WithResources for Rpc {
    #[tracing::instrument(skip_all)]
    async fn with_res(res: &Resources) -> anyhow::Result<Self> {
        let (ba, mfl, mcc, mcpk, rcc) = {
            let settings = res.read::<RpcSettings>();
            (
                settings.bind_address,
                settings.max_frame_length,
                settings.mpsc_channel_capacity,
                settings.max_channels_per_key,
                settings.rpc_channel_capacity,
            )
        };
        let receiver = res.write::<EventQueue<EngineEvent>>().subscribe::<Self>();

        let mut listener =
            tarpc::serde_transport::tcp::listen(&ba, tarpc::tokio_serde::formats::Bincode::default).await?;
        tracing::info!("RPC server listening on {}", listener.local_addr());
        listener.config_mut().max_frame_length(mfl);
        let (tx, rx) = mpsc::channel::<RpcMessage>(mcc);
        let rpc_listener: JoinHandle<()> = tokio::task::spawn(async move {
            tracing::trace!("Starting RPC listener");
            listener
                // Ignore accept errors.
                .filter_map(|r| ready(r.ok()))
                .map(BaseChannel::with_defaults)
                // Limit channels to 1 per IP and Port.
                .max_channels_per_key(mcpk, |t| t.transport().peer_addr().unwrap())
                // serve is generated by the service attribute. It takes as input any type implementing
                // the generated RpcService trait.
                .map(|channel| {
                    let connection = RpcServer::new(
                        tx.clone(),
                        channel.transport().peer_addr().unwrap(),
                    );
                    channel.execute(connection.serve()).for_each(|fut| async {
                        tokio::task::spawn(fut);
                    })
                })
                // Max 10 channels.
                .buffer_unordered(rcc)
                .for_each(|_| async { })
                .await;
        });

        Ok(Rpc {
            rpc_listener,
            mpsc_rx: rx,
            receiver,
        })
    }
}
