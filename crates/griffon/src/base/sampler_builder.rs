use crate::base::gpu_object_database::GpuObjectDatabase;
use crate::base::ids::SamplerId;
use crate::base::runtime::Runtime;

pub struct SamplerBuilder<'rt> {
    runtime: &'rt Runtime<'rt>,
    database: &'rt mut GpuObjectDatabase,
    label: Option<&'rt str>,
}

impl<'rt> SamplerBuilder<'rt> {
    pub(crate) fn new(runtime: &'rt Runtime, database: &'rt mut GpuObjectDatabase) -> Self {
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
        tracing::trace!("Creating sampler '{}'", self.label.unwrap_or("unnamed"));
        let sampler = self.runtime.device.create_sampler(&wgpu::SamplerDescriptor {
            label: self.label,
            ..Default::default()
        });
        self.database.insert_sampler(sampler)
    }
}
