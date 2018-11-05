pub mod camera;
pub mod info;
pub mod model;
pub mod renderable;

pub trait DepthOrderingTrait {
    fn depth_index(&self) -> i32;
}

pub trait TransformTrait: Sized {
    type Camera;

    fn transform(&self, camera: &Self::Camera, rhs: &Self) -> Option<Self>;
}
