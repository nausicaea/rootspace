use std::path::Path;
use anyhow::Result;
use thiserror::Error;

use super::AssetTrait;
use crate::graphics::vertex::Vertex;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn from_ply(_data: plyers::Ply) -> Result<Self, MeshError> {
        todo!()
    }
}

impl AssetTrait for Mesh {
    fn from_path<P: AsRef<Path>>(_path: P) -> Result<Self> {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum MeshError {
    #[error("The element '{0}' was not found")]
    ElementNotFound(&'static str),
    #[error("The property '{1}' was not found on element '{0}'")]
    PropertyNotFound(&'static str, &'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path() {
        let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply");

        let r: Result<Mesh> = Mesh::from_path(&p);
        assert!(r.is_ok());
    }
}
