use std::fmt;

pub mod camera;
pub mod info;
pub mod model;
pub mod renderable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    World,
    Ndc,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Layer::World => write!(f, "World"),
            Layer::Ndc => write!(f, "Ndc"),
        }
    }
}

pub trait DepthOrderingTrait {
    fn depth_index(&self) -> i32;
}
