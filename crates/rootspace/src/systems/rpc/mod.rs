pub mod graphics_info;
mod message;
mod server;
pub mod service;

use futures::StreamExt;
use futures::channel::{mpsc, oneshot};
use message::RpcMessage;
use std::thread::JoinHandle;
use std::{future::ready, time::Duration};
use tarpc::server::{BaseChannel, Channel, incoming::Incoming};
use tarpc::tokio_serde::formats::Bincode;
use tracing::error;

use crate::{
    assets::scene::Scene,
    events::engine_event::EngineEvent,
    resources::{rpc_settings::RpcSettings, statistics::Statistics},
    systems::rpc::{server::RpcServer, service::RpcService},
};
use assam::AssetDatabase;
use ecs::{EventQueue, ReceiverId, Resources, System, WithResources};
use graphics_info::GraphicsInfo;
use graphics_info::GraphicsInfoCategory;
use griffon::Graphics;

#[derive(Debug)]
pub struct Rpc {
    _rpc_listener: JoinHandle<anyhow::Result<()>>,
    mpsc_rx: mpsc::Receiver<RpcMessage>,
    abort_tx: Option<oneshot::Sender<()>>,
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
        let r = res.read::<AssetDatabase>().load_asset::<Scene, _>(res, group, name);
        if tx.send(r).is_err() {
            error!("unable to send the result of asset loading to the RPC server");
        }
    }

    #[tracing::instrument(skip_all)]
    async fn exit(&self, res: &Resources) {
        res.write::<EventQueue<EngineEvent>>().send(EngineEvent::Exit)
    }
}

impl System for Rpc {
    #[tracing::instrument(skip_all)]
    fn run(&mut self, res: &Resources, _t: Duration, _dt: Duration) {
        let events = res.write::<EventQueue<EngineEvent>>().receive(&self.receiver);
        for event in events {
            #[allow(irrefutable_let_patterns)]
            if let EngineEvent::Exit = event {
                tracing::trace!("Stopping RPC listener");
                if let Some(abort_tx) = self.abort_tx.take() {
                    let _ = abort_tx.send(());
                }
            }
        }

        while let Ok(Some(msg)) = self.mpsc_rx.try_next() {
            match msg {
                RpcMessage::StatsRequest(tx) => smol::block_on(self.perf(res, tx)),
                RpcMessage::GraphicsInfo { tx, category } => smol::block_on(self.graphics_info(res, tx, category)),
                RpcMessage::LoadScene {
                    tx,
                    ref group,
                    ref name,
                } => smol::block_on(self.load_scene(res, tx, group, name)),
                RpcMessage::Exit => smol::block_on(self.exit(res)),
            }
        }
    }
}

impl WithResources for Rpc {
    #[tracing::instrument(skip_all)]
    fn with_res(res: &Resources) -> anyhow::Result<Self> {
        let settings = res.read::<RpcSettings>().clone();
        let receiver = res.write::<EventQueue<EngineEvent>>().subscribe::<Self>();
        let (tx, rx) = mpsc::channel::<RpcMessage>(settings.mpsc_channel_capacity);
        let (abort_tx, abort_rx) = oneshot::channel::<()>();

        let rpc_listener = std::thread::spawn(move || smol::block_on(tarpc_thread(abort_rx, tx, &settings)));

        Ok(Rpc {
            _rpc_listener: rpc_listener,
            mpsc_rx: rx,
            abort_tx: Some(abort_tx),
            receiver,
        })
    }
}

#[tracing::instrument]
async fn tarpc_thread(
    mut abort_rx: oneshot::Receiver<()>,
    tx: mpsc::Sender<RpcMessage>,
    settings: &RpcSettings,
) -> anyhow::Result<()> {
    let mut listener = tarpc::serde_transport::tcp::listen(settings.bind_address, Bincode::default).await?;
    listener.config_mut().max_frame_length(settings.max_frame_length);

    tracing::info!("RPC server listening on {}", listener.local_addr());
    futures::select! {
        _ = &mut abort_rx => (),
        _ = listener
            // Ignore accept errors.
            .filter_map(|r| ready(r.ok()))
            .map(BaseChannel::with_defaults)
            // Limit channels to 1 per IP and Port.
            .max_channels_per_key(settings.max_channels_per_key, |t| t.transport().peer_addr().unwrap())
            // serve is generated by the service attribute. It takes as input any type implementing
            // the generated RpcService trait.
            .map(|channel| {
                let connection = RpcServer::new(
                    tx.clone(),
                    channel.transport().peer_addr().unwrap(),
                );
                channel.execute(connection.serve()).for_each(|fut| async {
                    // Spawn and detach a thread per connection
                    std::thread::spawn(|| smol::block_on(fut));
                })
            })
            // Max 10 channels.
            .buffer_unordered(settings.rpc_channel_capacity)
            .for_each(|_| async { }) => ()
    }

    Ok(())
}
