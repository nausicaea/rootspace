use glamour::mat::Mat4;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, align(256))]
pub struct CameraUniform(pub Mat4<f32>);

impl From<Mat4<f32>> for CameraUniform {
    fn from(value: Mat4<f32>) -> Self {
        CameraUniform(value)
    }
}
