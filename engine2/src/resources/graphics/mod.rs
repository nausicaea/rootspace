use ecs::{with_dependencies::WithDependencies, Resource};
use pollster::FutureExt;
use winit::event_loop::EventLoopWindowTarget;

use self::{
    render_pass_builder::RenderPassBuilder, render_pipeline_builder::RenderPipelineBuilder, runtime::Runtime,
    settings::Settings,
};

pub mod ids;
pub mod render_pass_builder;
pub mod render_pipeline_builder;
mod runtime;
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
}

impl Graphics {
    pub fn reconfigure(&mut self) {
        self.runtime.reconfigure()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.runtime.resize(new_size)
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.runtime.window.id()
    }

    pub fn create_render_pass(&self) -> RenderPassBuilder {
        self.runtime.create_render_pass(&self.settings)
    }

    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        self.runtime.create_render_pipeline()
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
