pub mod camera;
pub mod model;

use nalgebra::Matrix4;

pub trait AsMatrix {
    fn as_matrix(&self) -> &Matrix4<f32>;
}

pub trait DepthOrderingTrait {
    fn depth_index(&self) -> i32;
}
