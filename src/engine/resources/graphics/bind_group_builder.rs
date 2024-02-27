use super::{
    ids::{BindGroupId, BindGroupLayoutId, BufferId, SamplerId, TextureViewId},
    runtime::Runtime,
    Database,
};

pub struct BindGroupBuilder<'rt> {
    runtime: &'rt Runtime<'rt>,
    database: &'rt mut Database,
    layout: BindGroupLayoutId,
    label: Option<&'static str>,
    entries: Vec<(u32, BindingResourceId)>,
}

impl<'rt> BindGroupBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut Database, layout: BindGroupLayoutId) -> Self {
        BindGroupBuilder {
            runtime,
            database,
            layout,
            label: None,
            entries: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn add_buffer(mut self, binding: u32, offset: wgpu::BufferAddress, size: Option<wgpu::BufferSize>, buffer: BufferId) -> Self {
        self.entries.push((binding, BindingResourceId::Buffer { buffer, offset, size }));
        self
    }

    pub fn add_texture_view(mut self, binding: u32, view: TextureViewId) -> Self {
        self.entries.push((binding, BindingResourceId::TextureView(view)));
        self
    }

    pub fn add_sampler(mut self, binding: u32, sampler: SamplerId) -> Self {
        self.entries.push((binding, BindingResourceId::Sampler(sampler)));
        self
    }

    pub fn submit(self) -> BindGroupId {
        let layout = &self.database.bind_group_layouts[&self.layout];
        let entries: Vec<_> = self
            .entries
            .into_iter()
            .map(|(binding, r)| match r {
                BindingResourceId::Buffer { buffer, size, offset } => wgpu::BindGroupEntry {
                    binding,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.database.buffers[&buffer],
                        offset,
                        size,
                    }),
                },
                BindingResourceId::TextureView(v) => {
                    let view = &self.database.texture_views[&v];
                    wgpu::BindGroupEntry {
                        binding,
                        resource: wgpu::BindingResource::TextureView(view),
                    }
                }
                BindingResourceId::Sampler(s) => {
                    let sampler = &self.database.samplers[&s];
                    wgpu::BindGroupEntry {
                        binding,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    }
                }
            })
            .collect();

        log::trace!("Creating bind group '{}'", self.label.unwrap_or("unnamed"));
        let bg = self.runtime.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label,
            layout,
            entries: entries.as_slice(),
        });

        self.database.insert_bind_group(bg)
    }
}

#[derive(Debug, Clone, Copy)]
enum BindingResourceId {
    Buffer{ buffer: BufferId, offset: wgpu::BufferAddress, size: Option<wgpu::BufferSize> },
    TextureView(TextureViewId),
    Sampler(SamplerId),
}
