use super::{ids::BindGroupLayoutId, indexes::Indexes, runtime::Runtime, tables::Tables};

pub struct BindGroupLayoutBuilder<'rt, 'lbl> {
    runtime: &'rt Runtime,
    indexes: &'rt mut Indexes,
    tables: &'rt mut Tables,
    label: Option<&'lbl str>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl<'rt, 'lbl> BindGroupLayoutBuilder<'rt, 'lbl> {
    pub(super) fn new(runtime: &'rt Runtime, indexes: &'rt mut Indexes, tables: &'rt mut Tables) -> Self {
        Self {
            runtime,
            indexes,
            tables,
            label: None,
            entries: Vec::default(),
        }
    }

    pub fn with_label(mut self, label: &'lbl str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn add_bind_group_layout_entry(
        mut self,
        binding: u32,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BindingType,
    ) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        });
        self
    }

    pub fn submit(self, label: Option<&'static str>) -> BindGroupLayoutId {
        let bgl = self
            .runtime
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.label,
                entries: &self.entries,
            });

        let id = self.indexes.bind_group_layouts.take_labelled(label);
        log::trace!("Registering {:?} as {:?}", &bgl, id);
        self.tables.bind_group_layouts.insert(id, bgl);
        dbg!(&self.tables.bind_group_layouts);
        id
    }
}
