use super::{runtime::Runtime, settings::Settings, ids::PipelineId};

#[derive(Debug)]
pub struct RenderPassBuilder<'rt> {
    runtime: &'rt Runtime,
    settings: &'rt Settings,
    pipeline: Option<&'rt wgpu::RenderPipeline>,
}

impl<'rt> RenderPassBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, settings: &'rt Settings) -> Self {
        RenderPassBuilder { 
            runtime, 
            settings,
            pipeline: None,
        }
    }

    pub fn with_pipeline(mut self, pipeline: &PipelineId) -> Self {
        self.pipeline = Some(&self.runtime.tables.render_pipelines[pipeline]);
        self
    }

    pub fn submit(self, encoder_label: Option<&str>, pass_label: Option<&str>) -> Result<(), wgpu::SurfaceError> {
        let output = self.runtime.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.runtime.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: encoder_label,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: pass_label,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.settings.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            if let Some(p) = self.pipeline {
                render_pass.set_pipeline(p);
            }
        }

        // submit will accept anything that implements IntoIter
        let _si = self.runtime.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
