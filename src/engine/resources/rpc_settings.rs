use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use anyhow::Error;

use crate::ecs::{resource::Resource, with_dependencies::WithDependencies};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RpcSettings {
    pub bind_address: SocketAddr,
    pub max_frame_length: usize,
    pub mpsc_channel_capacity: usize,
    pub max_channels_per_key: u32,
    pub rpc_channel_capacity: usize,
}

impl Resource for RpcSettings {}

impl<D> WithDependencies<D> for RpcSettings
where
    D: RpcDeps + std::fmt::Debug,
{
    #[tracing::instrument]
    async fn with_deps(deps: &D) -> Result<Self, Error> {
        Ok(RpcSettings {
            bind_address: deps.bind_address(),
            max_frame_length: deps.max_frame_length(),
            mpsc_channel_capacity: deps.mpsc_channel_capacity(),
            max_channels_per_key: deps.max_channels_per_key(),
            rpc_channel_capacity: deps.rpc_channel_capacity(),
        })
    }
}

impl Default for RpcSettings {
    fn default() -> Self {
        RpcSettings {
            bind_address: (IpAddr::V6(Ipv6Addr::LOCALHOST), 7919).into(),
            max_frame_length: 8 * 1024 * 1024,
            mpsc_channel_capacity: 10,
            max_channels_per_key: 1,
            rpc_channel_capacity: 10,
        }
    }
}

pub trait RpcDeps {
    fn bind_address(&self) -> SocketAddr {
        (IpAddr::V6(Ipv6Addr::LOCALHOST), 7919).into()
    }
    fn max_frame_length(&self) -> usize {
        8 * 1024 * 1023
    }
    fn mpsc_channel_capacity(&self) -> usize {
        10
    }
    fn max_channels_per_key(&self) -> u32 {
        1
    }
    fn rpc_channel_capacity(&self) -> usize {
        10
    }
}
