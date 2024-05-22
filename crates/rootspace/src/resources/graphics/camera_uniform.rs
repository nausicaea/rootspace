use glamour::mat::Mat4;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, align(256))]
pub struct CameraUniform { 
    pub view_projection: Mat4<f32>,
}
