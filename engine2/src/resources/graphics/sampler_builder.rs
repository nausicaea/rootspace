use super::{ids::SamplerId, indexes::Indexes, runtime::Runtime, tables::Tables};

pub struct SamplerBuilder<'rt> {
    runtime: &'rt Runtime,
    indexes: &'rt mut Indexes,
    tables: &'rt mut Tables,
}

impl<'rt> SamplerBuilder<'rt> {
    pub fn new(runtime: &'rt Runtime, indexes: &'rt mut Indexes, tables: &'rt mut Tables) -> Self {
        Self {
            runtime,
            indexes,
            tables,
        }
    }

    pub fn submit(self) -> SamplerId {
        let sampler = self.runtime.device.create_sampler(&wgpu::SamplerDescriptor::default());
        let id = self.indexes.samplers.take();
        self.tables.samplers.insert(id, sampler);
        id
    }
}
