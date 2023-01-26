use std::ops::Range;

use super::{
    ids::{BindGroupId, BufferId, PipelineId},
    runtime::Runtime,
    settings::Settings,
    tables::Tables,
};

#[derive(Debug)]
pub struct RenderPassBuilder<'rt> {
    runtime: &'rt Runtime,
    settings: &'rt Settings,
    tables: &'rt Tables,
    pipeline: Option<PipelineId>,
    bind_groups: Vec<(u32, BindGroupId)>,
    vertex_buffers: Vec<(u32, BufferId)>,
    index_buffer: Option<BufferId>,
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
            bind_groups: Vec::new(),
            vertex_buffers: Vec::new(),
            index_buffer: None,
            draw_params: None,
            draw_indexed_params: None,
        }
    }

    pub fn with_pipeline(mut self, pipeline: PipelineId) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    pub fn add_vertex_buffer(mut self, slot: u32, buffer: BufferId) -> Self {
        self.vertex_buffers.push((slot, buffer));
        self
    }

    pub fn with_index_buffer(mut self, buffer: BufferId) -> Self {
        self.index_buffer = Some(buffer);
        self
    }

    pub fn add_bind_group(mut self, index: u32, group: BindGroupId) -> Self {
        self.bind_groups.push((index, group));
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
                render_pass.set_pipeline(&self.tables.render_pipelines[&p]);
            }

            for (i, bg) in self.bind_groups {
                render_pass.set_bind_group(i, &self.tables.bind_groups[&bg], &[]);
            }

            for (slot, vb) in self.vertex_buffers {
                render_pass.set_vertex_buffer(slot, self.tables.buffers[&vb].slice(..));
            }

            if let Some(ib) = self.index_buffer {
                render_pass.set_index_buffer(self.tables.buffers[&ib].slice(..), wgpu::IndexFormat::Uint32);
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
