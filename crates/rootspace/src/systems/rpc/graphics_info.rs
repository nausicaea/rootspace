use itertools::Itertools;
use std::fmt;
use std::ops::Range;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GraphicsInfo {
    InstanceReport(Box<Option<InstanceReport>>),
    SurfaceCapabilities(SurfaceCapabilities),
    AdapterFeatures(griffon::wgpu::Features),
    AdapterLimits(griffon::wgpu::Limits),
    AdapterDownlevelCapabilities(griffon::wgpu::DownlevelCapabilities),
    AdapterInfo(griffon::wgpu::AdapterInfo),
    DeviceAllocatorReport(Option<AllocatorReport>),
}

impl fmt::Display for GraphicsInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GraphicsInfo::InstanceReport(ir) => {
                if let Some(ir) = &**ir {
                    writeln!(f, "{ir}")
                } else {
                    writeln!(f, "no instance report available")
                }
            }
            GraphicsInfo::SurfaceCapabilities(sc) => {
                writeln!(f, "{sc}")
            }
            GraphicsInfo::AdapterFeatures(af) => {
                writeln!(f, "{af}")
            }
            GraphicsInfo::AdapterLimits(al) => {
                writeln!(f, "{al:?}")
            }
            GraphicsInfo::AdapterDownlevelCapabilities(adlc) => {
                writeln!(f, "{adlc:?}")
            }
            GraphicsInfo::AdapterInfo(ai) => {
                writeln!(f, "{ai:?}")
            }
            GraphicsInfo::DeviceAllocatorReport(Some(dar)) => {
                writeln!(f, "{dar:?}")
            }
            GraphicsInfo::DeviceAllocatorReport(None) => {
                writeln!(f, "no device allocator report available")
            }
        }
    }
}

/// Serializable proxy for [`griffon::wgpu_core::global::GlobalReport`](griffon::wgpu_core::global::GlobalReport)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InstanceReport {
    pub surfaces: RegistryReport,
    pub hub: HubReport,
}

impl From<griffon::wgpu_core::global::GlobalReport> for InstanceReport {
    fn from(r: griffon::wgpu_core::global::GlobalReport) -> Self {
        InstanceReport {
            surfaces: r.surfaces.into(),
            hub: r.hub.into(),
        }
    }
}

impl fmt::Display for InstanceReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Surfaces: {}\n{}", self.surfaces, self.hub,)
    }
}

/// Serializable proxy for [`griffon::wgpu_core::registry::RegistryReport`](griffon::wgpu_core::registry::RegistryReport)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RegistryReport {
    pub num_allocated: usize,
    pub num_kept_from_user: usize,
    pub num_released_from_user: usize,
    pub element_size: usize,
}

impl fmt::Display for RegistryReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "element size: {}, released from user: {}, kept from user: {}, total allocated: {}",
            self.element_size, self.num_released_from_user, self.num_kept_from_user, self.num_allocated,
        )
    }
}

impl From<griffon::wgpu_core::registry::RegistryReport> for RegistryReport {
    fn from(r: griffon::wgpu_core::registry::RegistryReport) -> Self {
        RegistryReport {
            num_allocated: r.num_allocated,
            num_kept_from_user: r.num_kept_from_user,
            num_released_from_user: r.num_released_from_user,
            element_size: r.element_size,
        }
    }
}

/// Serializable proxy for [`griffon::wgpu_core::HubReport`](griffon::wgpu_core::HubReport)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HubReport {
    pub adapters: RegistryReport,
    pub devices: RegistryReport,
    pub queues: RegistryReport,
    pub pipeline_layouts: RegistryReport,
    pub shader_modules: RegistryReport,
    pub bind_group_layouts: RegistryReport,
    pub bind_groups: RegistryReport,
    pub command_encoders: RegistryReport,
    pub command_buffers: RegistryReport,
    pub render_bundles: RegistryReport,
    pub render_pipelines: RegistryReport,
    pub compute_pipelines: RegistryReport,
    pub pipeline_caches: RegistryReport,
    pub query_sets: RegistryReport,
    pub buffers: RegistryReport,
    pub textures: RegistryReport,
    pub texture_views: RegistryReport,
    pub external_textures: RegistryReport,
    pub samplers: RegistryReport,
}

impl From<griffon::wgpu_core::hub::HubReport> for HubReport {
    fn from(r: griffon::wgpu_core::hub::HubReport) -> Self {
        HubReport {
            adapters: r.adapters.into(),
            devices: r.devices.into(),
            queues: r.queues.into(),
            pipeline_layouts: r.pipeline_layouts.into(),
            shader_modules: r.shader_modules.into(),
            bind_group_layouts: r.bind_group_layouts.into(),
            bind_groups: r.bind_groups.into(),
            command_encoders: r.command_encoders.into(),
            command_buffers: r.command_buffers.into(),
            render_bundles: r.render_bundles.into(),
            render_pipelines: r.render_pipelines.into(),
            compute_pipelines: r.compute_pipelines.into(),
            pipeline_caches: r.pipeline_caches.into(),
            query_sets: r.query_sets.into(),
            buffers: r.buffers.into(),
            textures: r.textures.into(),
            texture_views: r.texture_views.into(),
            external_textures: r.external_textures.into(),
            samplers: r.samplers.into(),
        }
    }
}

impl fmt::Display for HubReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Adapters: {}
Devices: {}
Queues: {}
Pipeline layouts: {}
Shader modules: {}
Bind group layouts: {}
Bind groups: {}
Command encoders: {}
Command buffers: {}
Render bundles: {}
Render pipelines: {}
Compute pipelines: {}
Pipeline caches: {}
Query sets: {}
Buffers: {}
Textures: {}
Texture views: {}
External textures: {}
Samplers: {}"#,
            self.adapters,
            self.devices,
            self.queues,
            self.pipeline_layouts,
            self.shader_modules,
            self.bind_group_layouts,
            self.bind_groups,
            self.command_encoders,
            self.command_buffers,
            self.render_bundles,
            self.render_pipelines,
            self.compute_pipelines,
            self.pipeline_caches,
            self.query_sets,
            self.buffers,
            self.textures,
            self.texture_views,
            self.external_textures,
            self.samplers,
        )
    }
}

/// Serializable proxy for [`griffon::wgpu::SurfaceCapabilities`](griffon::wgpu::SurfaceCapabilities)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SurfaceCapabilities {
    pub formats: Vec<griffon::wgpu_types::TextureFormat>,
    pub present_modes: Vec<griffon::wgpu_types::PresentMode>,
    pub alpha_modes: Vec<griffon::wgpu_types::CompositeAlphaMode>,
    pub usages: griffon::wgpu_types::TextureUsages,
}

impl From<griffon::wgpu::SurfaceCapabilities> for SurfaceCapabilities {
    fn from(r: griffon::wgpu::SurfaceCapabilities) -> Self {
        SurfaceCapabilities {
            formats: r.formats.into_iter().collect(),
            present_modes: r.present_modes.into_iter().collect(),
            alpha_modes: r.alpha_modes.into_iter().collect(),
            usages: r.usages,
        }
    }
}

impl fmt::Display for SurfaceCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Surface capabilities:
Texture formats: {}
Present modes: {}
Alpha modes: {}
Texture usages: {:?}"#,
            self.formats.iter().map(|f| format!("{f:?}")).join("\n  "),
            self.present_modes.iter().map(|pm| format!("{pm:?}")).join("\n  "),
            self.alpha_modes.iter().map(|am| format!("{am:?}")).join("\n  "),
            self.usages,
        )
    }
}

/// Serializable proxy for [`griffon::wgpu_types::AllocatorReport`](griffon::wgpu_types::AllocatorReport)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AllocatorReport {
    pub allocations: Vec<AllocationReport>,
    pub blocks: Vec<MemoryBlockReport>,
    pub total_allocated_bytes: u64,
    pub total_reserved_bytes: u64,
}

impl From<griffon::wgpu_types::AllocatorReport> for AllocatorReport {
    fn from(r: griffon::wgpu::AllocatorReport) -> Self {
        AllocatorReport {
            allocations: r.allocations.into_iter().map(Into::into).collect(),
            blocks: r.blocks.into_iter().map(Into::into).collect(),
            total_allocated_bytes: r.total_allocated_bytes,
            total_reserved_bytes: r.total_reserved_bytes,
        }
    }
}

impl fmt::Display for AllocatorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Allocator report:
Allocations: {}
Blocks: {}
Total allocated bytes: {}
Total reserved bytes: {}"#,
            self.allocations.iter().map(|a| format!("{a}")).join("\n  "),
            self.blocks.iter().map(|b| format!("{b}")).join("\n  "),
            self.total_allocated_bytes,
            self.total_reserved_bytes,
        )
    }
}

/// Serializable proxy for [`griffon::wgpu_types::AllocationReport`](griffon::wgpu_types::AllocationReport)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AllocationReport {
    pub name: String,
    pub offset: u64,
    pub size: u64,
}

impl From<griffon::wgpu_types::AllocationReport> for AllocationReport {
    fn from(r: griffon::wgpu_types::AllocationReport) -> Self {
        AllocationReport {
            name: r.name,
            offset: r.offset,
            size: r.size,
        }
    }
}

impl fmt::Display for AllocationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name: {}, offset: {}, size: {}", self.name, self.offset, self.size)
    }
}

/// Serializable proxy for [`griffon::wgpu_types::MemoryBlockReport`](griffon::wgpu_types::MemoryBlockReport)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoryBlockReport {
    pub size: u64,
    pub allocations: Range<usize>,
}

impl From<griffon::wgpu_types::MemoryBlockReport> for MemoryBlockReport {
    fn from(r: griffon::wgpu_types::MemoryBlockReport) -> Self {
        MemoryBlockReport {
            size: r.size,
            allocations: r.allocations,
        }
    }
}

impl fmt::Display for MemoryBlockReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "size: {}, allocations: {:?}", self.size, self.allocations)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum GraphicsInfoCategory {
    InstanceReport,
    SurfaceCapabilities,
    AdapterFeatures,
    AdapterLimits,
    AdapterDownlevelCapabilities,
    AdapterInfo,
    DeviceAllocatorReport,
}
