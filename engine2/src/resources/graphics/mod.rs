use ecs::{with_dependencies::WithDependencies, Resource};
use pollster::FutureExt;
use winit::event_loop::EventLoopWindowTarget;

use self::{
    render_pass_builder::RenderPassBuilder, render_pipeline_builder::RenderPipelineBuilder, runtime::Runtime,
    settings::Settings, ids::BindGroupLayoutId, indexes::Indexes, tables::Tables,
};

pub mod ids;
pub mod render_pass_builder;
pub mod render_pipeline_builder;
mod runtime;
mod indexes;
mod tables;
pub mod settings;
mod urn;

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
            self.runtime.surface.configure(&self.runtime.device, &self.runtime.config);
        }
    }

    pub fn get_bind_group_layout(&self, bgl: &BindGroupLayoutId) -> &wgpu::BindGroupLayout {
        &self.tables.bind_group_layouts[bgl]
    }

    pub fn create_render_pass(&self) -> RenderPassBuilder {
        RenderPassBuilder::new(&self.runtime, &self.settings, &self.tables)
    }

    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        RenderPipelineBuilder::new(&mut self.runtime, &mut self.indexes, &mut self.tables)
    }

    pub fn create_bind_group_layout(
        &mut self,
        label: Option<&str>,
        entries: &[wgpu::BindGroupLayoutEntry],
    ) -> BindGroupLayoutId {
        let bgl = self
            .runtime
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label, entries });

        let id = self.indexes.bind_group_layouts.take();
        self.tables.bind_group_layouts.insert(id, bgl);
        id
    }

    //pub fn create_bind_group(&mut self)
    //pub fn create_buffer(&mut self)
    //pub fn create_texture(&mut self)
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
