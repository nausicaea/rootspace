use std::{
    convert::{TryFrom, TryInto},
    path::Path,
};

use anyhow::Result;
use file_manipulation::FilePathBuf;
use thiserror::Error;

use super::AssetTrait;
use crate::graphics::vertex::Vertex;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn from_ply(data: &ply::Ply) -> Result<Self, MeshError> {
        let (eidx, el) = data
            .element(&["vertex", "vertices"])
            .ok_or(MeshError::ElementNotFound("vertex"))?;
        let (pos_x_idx, _) = el
            .scalar_property(&["x", "pos_x"])
            .ok_or(MeshError::PropertyNotFound("vertex", "x"))?;
        let (pos_y_idx, _) = el
            .scalar_property(&["y", "pos_y"])
            .ok_or(MeshError::PropertyNotFound("vertex", "y"))?;
        let (pos_z_idx, _) = el
            .scalar_property(&["z", "pos_z"])
            .ok_or(MeshError::PropertyNotFound("vertex", "z"))?;
        let (tex_u_idx, _) = el
            .scalar_property(&["s", "u", "tex_u"])
            .ok_or(MeshError::PropertyNotFound("vertex", "s"))?;
        let (tex_v_idx, _) = el
            .scalar_property(&["t", "v", "tex_v"])
            .ok_or(MeshError::PropertyNotFound("vertex", "t"))?;
        let (norm_x_idx, _) = el
            .scalar_property(&["nx", "norm_x"])
            .ok_or(MeshError::PropertyNotFound("vertex", "nx"))?;
        let (norm_y_idx, _) = el
            .scalar_property(&["ny", "norm_y"])
            .ok_or(MeshError::PropertyNotFound("vertex", "ny"))?;
        let (norm_z_idx, _) = el
            .scalar_property(&["nz", "norm_z"])
            .ok_or(MeshError::PropertyNotFound("vertex", "nz"))?;

        let vertices = data.generate(eidx, |props| {
            let p = [
                (&props[pos_x_idx]).try_into().unwrap(),
                (&props[pos_y_idx]).try_into().unwrap(),
                (&props[pos_z_idx]).try_into().unwrap(),
            ];
            let t = [
                (&props[tex_u_idx]).try_into().unwrap(),
                (&props[tex_v_idx]).try_into().unwrap(),
            ];
            let n = [
                (&props[norm_x_idx]).try_into().unwrap(),
                (&props[norm_y_idx]).try_into().unwrap(),
                (&props[norm_z_idx]).try_into().unwrap(),
            ];
            Vertex::new(p, t, n)
        });

        let (eidx, el) = data
            .element(&["face", "faces"])
            .ok_or(MeshError::ElementNotFound("face"))?;
        let (idx, _) = el
            .vector_property(&["vertex_indices", "vertex_index"])
            .ok_or(MeshError::PropertyNotFound("face", "vertex_indices"))?;

        let indices = data
            .generate(eidx, |p| TryInto::<Vec<u16>>::try_into(&p[idx]).unwrap())
            .into_iter()
            .flatten()
            .collect();

        Ok(Mesh { vertices, indices })
    }
}

impl AssetTrait for Mesh {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let fp = FilePathBuf::try_from(path.as_ref())?;
        let data = ply::Ply::from_path(fp)?;
        let mesh = Mesh::from_ply(&data)?;

        Ok(mesh)
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
