use super::{ids::SamplerId, runtime::Runtime, Database};

pub struct SamplerBuilder<'rt> {
    runtime: &'rt Runtime<'rt>,
    database: &'rt mut Database,
    label: Option<&'rt str>,
}

impl<'rt> SamplerBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut Database) -> Self {
        Self {
            runtime,
            database,
            label: None,
        }
    }

    pub fn with_label(mut self, label: Option<&'rt str>) -> Self {
        self.label = label;
        self
    }

    pub fn submit(self) -> SamplerId {
        log::trace!("Creating sampler '{}'", self.label.unwrap_or("unnamed"));
        let sampler = self.runtime.device.create_sampler(&wgpu::SamplerDescriptor {
            label: self.label,
            ..Default::default()
        });
        self.database.insert_sampler(sampler)
    }
}
