use super::{
    ids::{BindGroupId, BindGroupLayoutId},
    indexes::Indexes,
    runtime::Runtime,
    tables::Tables,
};

pub struct BindGroupBuilder<'rt, 'lbl, 'bge> {
    runtime: &'rt Runtime,
    indexes: &'rt mut Indexes,
    tables: &'rt mut Tables,
    layout: BindGroupLayoutId,
    label: Option<&'lbl str>,
    entries: Vec<wgpu::BindGroupEntry<'bge>>,
}

impl<'rt, 'lbl, 'bge> BindGroupBuilder<'rt, 'lbl, 'bge> {
    pub(super) fn new(
        runtime: &'rt Runtime,
        indexes: &'rt mut Indexes,
        tables: &'rt mut Tables,
        layout: BindGroupLayoutId,
    ) -> Self {
        BindGroupBuilder {
            runtime,
            indexes,
            tables,
            layout,
            label: None,
            entries: Vec::new(),
        }
    }

    pub fn with_label(mut self, label: &'lbl str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn add_bind_group_entry(mut self, binding: u32, resource: wgpu::BindingResource) -> Self {
        self.entries.push(wgpu::BindGroupEntry { binding, resource });
        self
    }

    pub fn submit(self) -> BindGroupId {
        let layout = &self.tables.bind_group_layouts[&self.layout];
        let bg = self.runtime.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label,
            layout,
            entries: &self.entries,
        });

        let id = self.indexes.bind_groups.take();
        self.tables.bind_groups.insert(id, bg);
        id
    }
}
