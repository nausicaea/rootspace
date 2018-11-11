pub mod camera;
pub mod info;
pub mod model;
pub mod renderable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    World,
    Ndc,
}

pub trait DepthOrderingTrait {
    fn depth_index(&self) -> i32;
}

pub trait TransformTrait: Sized {
    type Camera;

    fn layer(&self) -> Layer;
    fn transform(&self, camera: &Self::Camera, rhs: &Self) -> Option<Self>;
}
