use std::ops::Range;

use wgpu::{LoadOp, StoreOp};

use super::{
    ids::{BindGroupId, BufferId, PipelineId},
    runtime::Runtime,
    settings::Settings,
    Database,
};
use crate::engine::resources::graphics::ids::TextureViewId;

#[derive(Debug)]
pub struct Encoder<'rt> {
    runtime: &'rt Runtime<'rt>,
    settings: &'rt Settings,
    database: &'rt Database,
    depth_texture_view: TextureViewId,
    output: wgpu::SurfaceTexture,
    surface_view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl<'rt> Encoder<'rt> {
    pub(super) fn new(
        label: Option<&str>,
        runtime: &'rt Runtime,
        settings: &'rt Settings,
        database: &'rt Database,
        depth_texture_view: TextureViewId,
    ) -> Result<Self, wgpu::SurfaceError> {
        crate::trace_gfx!("Getting surface texture");
        let output = runtime.surface.get_current_texture()?;

        crate::trace_gfx!("Creating surface texture view '{}'", label.unwrap_or("unnamed"));
        let surface_view = output.texture.create_view(&wgpu::TextureViewDescriptor {
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
            depth_texture_view,
            output,
            surface_view,
            encoder,
        })
    }

    pub fn begin(&mut self, label: Option<&str>) -> RenderPass {
        crate::trace_gfx!("Obtain ref. for depth texture view");
        let dtv = self
            .database
            .texture_views
            .get(&self.depth_texture_view)
            .unwrap_or_else(|| {
                panic!(
                    "Developer error: found no depth texture with ID {:?}",
                    self.depth_texture_view
                )
            });

        crate::trace_gfx!("Beginning render pass '{}'", label.unwrap_or("unnamed"));
        let render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: LoadOp::Clear(self.settings.clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: dtv,
                depth_ops: Some(wgpu::Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        RenderPass {
            render_pass,
            database: self.database,
        }
    }

    pub fn submit(self) {
        crate::trace_gfx!("Creating command buffer");
        let command_buffer = self.encoder.finish();
        crate::trace_gfx!("Submitting command buffer");
        #[allow(unused_variables)]
        let si = self.runtime.queue.submit(std::iter::once(command_buffer));
        crate::trace_gfx!("Submission index: {:?}", si);
        self.runtime.window.pre_present_notify();
        self.output.present();
    }
}

#[derive(Debug)]
pub struct RenderPass<'rp> {
    render_pass: wgpu::RenderPass<'rp>,
    database: &'rp Database,
}

impl<'rp> RenderPass<'rp> {
    pub fn set_pipeline(&mut self, pipeline: PipelineId) -> &mut Self {
        self.render_pass
            .set_pipeline(&self.database.render_pipelines[&pipeline]);
        self
    }

    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: BindGroupId,
        offsets: &[wgpu::DynamicOffset],
    ) -> &mut Self {
        self.render_pass
            .set_bind_group(index, &self.database.bind_groups[&bind_group], offsets);
        self
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: BufferId) -> &mut Self {
        self.render_pass
            .set_vertex_buffer(slot, self.database.buffers[&buffer].slice(..));
        self
    }

    pub fn set_index_buffer(&mut self, buffer: BufferId) -> &mut Self {
        self.render_pass
            .set_index_buffer(self.database.buffers[&buffer].slice(..), wgpu::IndexFormat::Uint32);
        self
    }

    pub fn draw_indexed(&mut self, ind: Range<u32>, base_vert: i32, inst: Range<u32>) -> &mut Self {
        self.render_pass.draw_indexed(ind, base_vert, inst);
        self
    }
}
