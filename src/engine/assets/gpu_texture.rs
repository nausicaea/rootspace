use super::cpu_texture::CpuTexture;
use crate::{
    ecs::resources::Resources,
    engine::resources::graphics::{
        ids::{SamplerId, TextureId, TextureViewId},
        Graphics,
    },
};

#[derive(Debug)]
pub struct GpuTexture {
    pub texture: TextureId,
    pub view: TextureViewId,
    pub sampler: SamplerId,
}

impl GpuTexture {
    pub fn with_texture(res: &Resources, t: &CpuTexture) -> Self {
        let mut gfx = res.write::<Graphics>();
        let texture = gfx
            .create_texture()
            .with_label(t.label.as_ref().map(|l| format!("{}:texture", &l)).as_deref())
            .with_image(&t.image)
            .submit();
        let view = gfx.create_texture_view(
            t.label.as_ref().map(|l| format!("{}:texture-view", &l)).as_deref(),
            texture,
        );
        let sampler = gfx
            .create_sampler()
            .with_label(t.label.as_ref().map(|l| format!("{}:texture-sampler", &l)).as_deref())
            .submit();

        GpuTexture { texture, view, sampler }
    }
}
