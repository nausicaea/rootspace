use std::ops::Range;

use super::{
    ids::{BindGroupId, BufferId, PipelineId},
    runtime::Runtime,
    settings::Settings,
    Database,
};

#[derive(Debug)]
pub struct Encoder<'rt> {
    runtime: &'rt Runtime,
    settings: &'rt Settings,
    database: &'rt Database,
    output: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl<'rt> Encoder<'rt> {
    pub(super) fn new(
        runtime: &'rt Runtime,
        settings: &'rt Settings,
        database: &'rt Database,
    ) -> Result<Self, wgpu::SurfaceError> {
        let output = runtime.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        Ok(Encoder {
            runtime,
            settings,
            database,
            output,
            view,
            encoder,
        })
    }

    pub fn begin(&mut self) -> RenderPass {
        let render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.settings.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        RenderPass(render_pass, &self.database)
    }

    pub fn submit(self) {
        let _si = self.runtime.queue.submit(std::iter::once(self.encoder.finish()));
        self.output.present();
    }
}

#[derive(Debug)]
pub struct RenderPass<'rp>(wgpu::RenderPass<'rp>, &'rp Database);

impl<'rp> RenderPass<'rp> {
    pub fn set_pipeline(&mut self, pipeline: PipelineId) -> &mut Self {
        self.0.set_pipeline(&self.1.render_pipelines[&pipeline]);
        self
    }

    pub fn set_bind_group(&mut self, index: u32, bind_group: BindGroupId) -> &mut Self {
        self.0.set_bind_group(index, &self.1.bind_groups[&bind_group], &[]);
        self
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: BufferId) -> &mut Self {
        self.0.set_vertex_buffer(slot, self.1.buffers[&buffer].slice(..));
        self
    }

    pub fn set_index_buffer(&mut self, buffer: BufferId) -> &mut Self {
        self.0
            .set_index_buffer(self.1.buffers[&buffer].slice(..), wgpu::IndexFormat::Uint32);
        self
    }

    pub fn draw(&mut self, vert: Range<u32>, inst: Range<u32>) -> &mut Self {
        self.0.draw(vert, inst);
        self
    }

    pub fn draw_indexed(&mut self, ind: Range<u32>, base_vert: i32, inst: Range<u32>) -> &mut Self {
        self.0.draw_indexed(ind, base_vert, inst);
        self
    }
}
