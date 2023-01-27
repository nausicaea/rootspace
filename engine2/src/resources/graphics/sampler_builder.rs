use super::{ids::SamplerId, runtime::Runtime, Database};

pub struct SamplerBuilder<'rt> {
    runtime: &'rt Runtime,
    database: &'rt mut Database,
}

impl<'rt> SamplerBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, database: &'rt mut Database) -> Self {
        Self { runtime, database }
    }

    pub fn submit(self) -> SamplerId {
        let sampler = self.runtime.device.create_sampler(&wgpu::SamplerDescriptor::default());
        self.database.insert_sampler(None, sampler)
    }
}
