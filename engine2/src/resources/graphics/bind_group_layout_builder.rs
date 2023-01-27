use super::{ids::BindGroupLayoutId, runtime::Runtime, Database};

pub struct BindGroupLayoutBuilder<'rt, 'lbl> {
    runtime: &'rt Runtime,
    database: &'rt mut Database,
    label: Option<&'lbl str>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl<'rt, 'lbl> BindGroupLayoutBuilder<'rt, 'lbl> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut Database) -> Self {
        Self {
            runtime,
            database,
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

        self.database.insert_bind_group_layout(label, bgl)
    }
}
