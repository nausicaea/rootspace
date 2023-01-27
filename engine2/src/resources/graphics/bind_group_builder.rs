use super::{
    ids::{BindGroupId, BindGroupLayoutId, BufferId, SamplerId, TextureViewId},
    runtime::Runtime,
    Database,
};

pub struct BindGroupBuilder<'rt, 'lbl> {
    runtime: &'rt Runtime,
    database: &'rt mut Database,
    layout: BindGroupLayoutId,
    label: Option<&'lbl str>,
    entries: Vec<(u32, BindingResourceId)>,
}

impl<'rt, 'lbl> BindGroupBuilder<'rt, 'lbl> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut Database, layout: BindGroupLayoutId) -> Self {
        BindGroupBuilder {
            runtime,
            database,
            layout,
            label: None,
            entries: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: &'lbl str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn add_buffer(mut self, binding: u32, buffer: BufferId) -> Self {
        self.entries.push((binding, BindingResourceId::Buffer(buffer)));
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
                BindingResourceId::Buffer(b) => {
                    let buf = self.database.buffers[&b].as_entire_binding();
                    wgpu::BindGroupEntry { binding, resource: buf }
                }
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
        let bg = self.runtime.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label,
            layout,
            entries: entries.as_slice(),
        });

        self.database.insert_bind_group(None, bg)
    }
}

#[derive(Debug, Clone, Copy)]
enum BindingResourceId {
    Buffer(BufferId),
    TextureView(TextureViewId),
    Sampler(SamplerId),
}
