use glamour::mat::Mat4;

#[repr(C, align(256))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraUniform(pub Mat4<f32>);

impl From<Mat4<f32>> for CameraUniform {
    fn from(value: Mat4<f32>) -> Self {
        CameraUniform(value)
    }
}

