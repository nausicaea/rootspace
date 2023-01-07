use ecs::{with_dependencies::WithDependencies, Resource};
use pollster::FutureExt;
use winit::event_loop::EventLoopWindowTarget;

use self::{
    bind_group_layout_builder::BindGroupLayoutBuilder,
    ids::{BindGroupId, BindGroupLayoutId},
    indexes::Indexes,
    render_pass_builder::RenderPassBuilder,
    render_pipeline_builder::RenderPipelineBuilder,
    runtime::Runtime,
    settings::Settings,
    tables::Tables,
};

pub mod bind_group_layout_builder;
pub mod ids;
mod indexes;
pub mod render_pass_builder;
pub mod render_pipeline_builder;
mod runtime;
pub mod settings;
mod tables;
mod urn;
pub mod vertex_attribute_descriptor;

pub trait GraphicsDeps {
    type CustomEvent: 'static;

    fn event_loop(&self) -> &EventLoopWindowTarget<Self::CustomEvent>;
    fn settings(&self) -> &Settings;
}

#[derive(Debug)]
pub struct Graphics {
    settings: Settings,
    runtime: Runtime,
    indexes: Indexes,
    tables: Tables,
}

impl Graphics {
    pub fn window_id(&self) -> winit::window::WindowId {
        self.runtime.window.id()
    }

    pub fn reconfigure(&mut self) {
        self.resize(self.runtime.size)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.runtime.size = new_size;
            self.runtime.config.width = new_size.width;
            self.runtime.config.height = new_size.height;
            self.runtime
                .surface
                .configure(&self.runtime.device, &self.runtime.config);
        }
    }

    pub fn create_render_pass(&self) -> RenderPassBuilder {
        RenderPassBuilder::new(&self.runtime, &self.settings, &self.tables)
    }

    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        RenderPipelineBuilder::new(&self.runtime, &mut self.indexes, &mut self.tables)
    }

    pub fn create_bind_group_layout(&mut self) -> BindGroupLayoutBuilder {
        BindGroupLayoutBuilder::new(&self.runtime, &mut self.indexes, &mut self.tables)
    }

    pub fn create_bind_group(&mut self) -> BindGroupBuilder {
        BindGroupBuilder::new(&self.runtime, &mut self.indexes, &mut self.tables)
    }

    //pub fn create_buffer(&mut self)
    //pub fn create_texture(&mut self)
    //pub fn create_sampler(&mut self)
}

pub struct BindGroupBuilder<'rt, 'bgl, 'bge> {
    runtime: &'rt Runtime,
    indexes: &'rt mut Indexes,
    tables: &'rt mut Tables,
    layout: Option<&'bgl BindGroupLayoutId>,
    entries: Vec<wgpu::BindGroupEntry<'bge>>,
}

impl<'rt, 'bgl, 'bge> BindGroupBuilder<'rt, 'bgl, 'bge> {
    pub(super) fn new(runtime: &'rt Runtime, indexes: &'rt mut Indexes, tables: &'rt mut Tables) -> Self {
        BindGroupBuilder {
            runtime,
            indexes,
            tables,
            layout: None,
            entries: Vec::new(),
        }
    }

    pub fn with_layout(mut self, layout: &'bgl BindGroupLayoutId) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn add_entry(mut self, entry: wgpu::BindGroupEntry<'bge>) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn submit(self, label: Option<&str>) -> BindGroupId {
        let layout_id = self.layout.expect("cannot build a bind group without a layout");
        let layout = &self.tables.bind_group_layouts[layout_id];
        let bg = self.runtime.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout,
            entries: &self.entries,
        });

        let id = self.indexes.bind_groups.take();
        self.tables.bind_groups.insert(id, bg);
        id
    }
}

impl Resource for Graphics {}

impl<D: GraphicsDeps> WithDependencies<D> for Graphics {
    fn with_deps(deps: &D) -> Result<Self, anyhow::Error> {
        let settings = deps.settings();
        let runtime = Runtime::new(
            deps.event_loop(),
            settings.backends,
            settings.power_preference,
            settings.features,
            settings.limits.clone(),
            &settings.preferred_texture_format,
            settings.present_mode,
            settings.alpha_mode,
        )
        .block_on();

        Ok(Graphics {
            settings: settings.clone(),
            runtime,
            indexes: Indexes::default(),
            tables: Tables::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use ecs::Reg;

    use super::*;

    #[test]
    fn graphics_reg_macro() {
        type _RR = Reg![Graphics];
    }
}
