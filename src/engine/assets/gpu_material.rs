use crate::engine::resources::graphics::ids::BindGroupId;
use crate::engine::resources::graphics::Graphics;

use super::{cpu_material::CpuMaterial, gpu_texture::GpuTexture};

#[derive(Debug)]
pub struct GpuMaterial {
    pub texture: GpuTexture,
    pub bind_group: BindGroupId,
}

impl GpuMaterial {
    pub fn with_material(res: &crate::ecs::resources::Resources, m: &CpuMaterial) -> Self {
        let texture = GpuTexture::with_texture(res, &m.texture);

        let mut gfx = res.write::<Graphics>();
        let layout = gfx.material_layout();
        let bind_group = gfx
            .create_bind_group(layout)
            .with_label(m.label.as_ref().map(|l| format!("{}:bind-group", &l)).as_deref())
            .add_texture_view(0, texture.view)
            .add_sampler(1, texture.sampler)
            .submit();

        GpuMaterial { texture, bind_group }
    }
}

