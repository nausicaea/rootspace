pub mod image;
pub mod mesh;
pub mod text;
pub mod vertex;

use std::path::Path;
use failure::Error;
pub use self::{image::Image, mesh::Mesh, text::Text, vertex::Vertex};

pub trait AssetTrait: Sized {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}
