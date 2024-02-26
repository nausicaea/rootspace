use std::ops::Range;
use wgpu::StoreOp;

use super::{
    ids::{BindGroupId, BufferId, PipelineId},
    runtime::Runtime,
    settings::Settings,
    Database,
};

#[derive(Debug)]
pub struct Encoder<'rt> {
    runtime: &'rt Runtime<'rt>,
    settings: &'rt Settings,
    database: &'rt Database,
    output: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl<'rt> Encoder<'rt> {
    pub(super) fn new(
        label: Option<&str>,
        runtime: &'rt Runtime,
        settings: &'rt Settings,
        database: &'rt Database,
    ) -> Result<Self, wgpu::SurfaceError> {
        crate::trace_gfx!("Getting surface texture");
        let output = runtime.surface.get_current_texture()?;

        crate::trace_gfx!("Creating surface texture view '{}'", label.unwrap_or("unnamed"));
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: label.map(|lbl| format!("{}:surface-texture-view", lbl)).as_deref(),
            ..Default::default()
        });

        crate::trace_gfx!("Creating command encoder '{}'", label.unwrap_or("unnamed"));
        let encoder = runtime
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label });

        Ok(Encoder {
            runtime,
            settings,
            database,
            output,
            view,
            encoder,
        })
    }

    pub fn begin(&mut self, label: Option<&str>) -> RenderPass {
        crate::trace_gfx!("Beginning render pass '{}'", label.unwrap_or("unnamed"));
        let render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.settings.clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        RenderPass(render_pass, &self.database)
    }

    pub fn submit(self) {
        crate::trace_gfx!("Submitting command encoder");
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

    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: BindGroupId,
        offsets: &[wgpu::DynamicOffset],
    ) -> &mut Self {
        self.0.set_bind_group(index, &self.1.bind_groups[&bind_group], offsets);
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
