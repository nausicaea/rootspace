pub mod camera;
pub mod model;

pub trait DepthOrderingTrait {
    fn depth_index(&self) -> i32;
}
