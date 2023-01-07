use super::{ids::BindGroupLayoutId, runtime::Runtime, indexes::Indexes, tables::Tables};

pub struct BindGroupLayoutBuilder<'rt> {
    runtime: &'rt Runtime,
    indexes: &'rt mut Indexes,
    tables: &'rt mut Tables,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl<'rt> BindGroupLayoutBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, indexes: &'rt mut Indexes, tables: &'rt mut Tables) -> Self {
        Self {
            runtime, indexes, tables, entries: Vec::default(),
        }
    }

    pub fn add_bind_group_layout_entry(mut self, binding: u32, visibility: wgpu::ShaderStages, ty: wgpu::BindingType) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        });
        self
    }

    pub fn submit(self, label: Option<&str>) -> BindGroupLayoutId {
        let bgl = self
            .runtime
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label, entries: &self.entries });

        let id = self.indexes.bind_group_layouts.take();
        self.tables.bind_group_layouts.insert(id, bgl);
        id
    }
}
