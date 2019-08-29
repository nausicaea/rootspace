pub mod image;
pub mod mesh;
pub mod text;
pub mod vertex;

pub use self::{image::Image, mesh::Mesh, text::Text, vertex::Vertex};
use failure::Error;
use std::path::Path;

pub trait AssetTrait: Sized {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}
