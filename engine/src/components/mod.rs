pub mod camera;
pub mod model;
pub mod renderable;

pub trait DepthOrderingTrait {
    fn depth_index(&self) -> i32;
}
