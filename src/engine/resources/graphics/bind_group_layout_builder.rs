use super::{ids::BindGroupLayoutId, runtime::Runtime, Database};

pub struct BindGroupLayoutBuilder<'rt> {
    runtime: &'rt Runtime<'rt>,
    database: &'rt mut Database,
    label: Option<&'static str>,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl<'rt> BindGroupLayoutBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut Database) -> Self {
        Self {
            runtime,
            database,
            label: None,
            entries: Vec::default(),
        }
    }

    pub fn with_label(mut self, label: &'static str) -> Self {
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

    pub fn submit(self) -> BindGroupLayoutId {
        log::trace!("Creating bind group layout '{}'", self.label.unwrap_or("unnamed"));
        let bgl = self
            .runtime
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.label,
                entries: &self.entries,
            });

        self.database.insert_bind_group_layout(bgl)
    }
}
