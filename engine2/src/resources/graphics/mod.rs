use ecs::{Resource, SerializationName};
use serde::{Deserialize, Serialize};
use winit::{event_loop::EventLoopWindowTarget, window::WindowId};

use self::{runtime::Runtime, settings::Settings, render_pipeline_builder::RenderPipelineBuilder};

pub mod ids;
pub mod render_pipeline_builder;
mod runtime;
mod settings;
mod urn;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Graphics {
    settings: Settings,
    #[serde(skip)]
    runtime: Option<Runtime>,
}

impl Graphics {
    pub async fn initialize<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
        self.runtime = Some(
            Runtime::new(
                event_loop,
                self.settings.backends,
                self.settings.power_preference,
                self.settings.features,
                self.settings.limits.clone(),
                &self.settings.preferred_texture_format,
                self.settings.present_mode,
                self.settings.alpha_mode,
            )
            .await,
        );
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(ref mut rt) = self.runtime {
            rt.resize(new_size)
        }
    }

    pub fn render<F>(&self, rdr: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&mut wgpu::RenderPass),
    {
        let rt = self.runtime.as_ref().unwrap();

        rt.render(self.settings.clear_color, rdr)?;

        Ok(())
    }

    pub fn window_id(&self) -> Option<WindowId> {
        self.runtime.as_ref().map(|rt| rt.window.id())
    }

    pub fn create_render_pipeline(&mut self) -> RenderPipelineBuilder {
        let rt = self.runtime.as_mut().unwrap();
        rt.create_render_pipeline()
    }
}

impl Resource for Graphics {}

impl SerializationName for Graphics {}
