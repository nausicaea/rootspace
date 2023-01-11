use std::ops::Range;

use super::{
    ids::{BufferId, PipelineId},
    runtime::Runtime,
    settings::Settings,
    tables::Tables,
};

#[derive(Debug)]
pub struct RenderPassBuilder<'rt> {
    runtime: &'rt Runtime,
    settings: &'rt Settings,
    tables: &'rt Tables,
    pipeline: Option<&'rt wgpu::RenderPipeline>,
    vertex_buffer: Option<(u32, &'rt wgpu::Buffer)>,
    index_buffer: Option<&'rt wgpu::Buffer>,
    draw_params: Option<(Range<u32>, Range<u32>)>,
    draw_indexed_params: Option<(Range<u32>, i32, Range<u32>)>,
}

impl<'rt> RenderPassBuilder<'rt> {
    pub(super) fn new(runtime: &'rt Runtime, settings: &'rt Settings, tables: &'rt Tables) -> Self {
        RenderPassBuilder {
            runtime,
            settings,
            tables,
            pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            draw_params: None,
            draw_indexed_params: None,
        }
    }

    pub fn with_pipeline(mut self, pipeline: &PipelineId) -> Self {
        self.pipeline = Some(&self.tables.render_pipelines[pipeline]);
        self
    }

    pub fn with_vertex_buffer(mut self, slot: u32, buffer: &BufferId) -> Self {
        self.vertex_buffer = Some((slot, &self.tables.buffers[buffer]));
        self
    }

    pub fn with_index_buffer(mut self, buffer: &BufferId) -> Self {
        self.index_buffer = Some(&self.tables.buffers[buffer]);
        self
    }

    pub fn draw(mut self, vertices: Range<u32>, instances: Range<u32>) -> Self {
        self.draw_params = Some((vertices, instances));
        self
    }

    pub fn draw_indexed(mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) -> Self {
        self.draw_indexed_params = Some((indices, base_vertex, instances));
        self
    }

    pub fn submit(self, encoder_label: Option<&str>, pass_label: Option<&str>) -> Result<(), wgpu::SurfaceError> {
        let output = self.runtime.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: encoder_label });

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

            if let Some((slot, vb)) = self.vertex_buffer {
                render_pass.set_vertex_buffer(slot, vb.slice(..));
            }

            if let Some(ib) = self.index_buffer {
                render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
            }

            if let Some((vert, inst)) = self.draw_params {
                render_pass.draw(vert, inst);
            }

            if let Some((ind, base_vert, inst)) = self.draw_indexed_params {
                render_pass.draw_indexed(ind, base_vert, inst);
            }
        }

        // submit will accept anything that implements IntoIter
        let _si = self.runtime.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
