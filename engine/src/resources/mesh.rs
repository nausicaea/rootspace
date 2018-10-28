use super::vertex::Vertex;
use failure::Error;
use file_manipulation::VerifyPath;
use ply::{self, CoerceTo};
use std::path::Path;

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        path.ensure_extant_file()?;
        let data = ply::Ply::from_path(path)?;
        let mesh = Mesh::from_ply(&data)?;

        Ok(mesh)
    }

    pub fn from_ply(data: &ply::Ply) -> Result<Self, MeshError> {
        let vtx_keyword = "vertex";
        let (eidx, el) = data
            .element(&[vtx_keyword, "vertices"])
            .ok_or(MeshError::ElementNotFound(vtx_keyword))?;
        let (pos_x_idx, _) = el
            .scalar_property(&["x", "pos_x"])
            .ok_or(MeshError::PropertyNotFound(vtx_keyword, "x"))?;
        let (pos_y_idx, _) = el
            .scalar_property(&["y", "pos_y"])
            .ok_or(MeshError::PropertyNotFound(vtx_keyword, "y"))?;
        let (pos_z_idx, _) = el
            .scalar_property(&["z", "pos_z"])
            .ok_or(MeshError::PropertyNotFound(vtx_keyword, "z"))?;
        let (tex_u_idx, _) = el
            .scalar_property(&["s", "u", "tex_u"])
            .ok_or(MeshError::PropertyNotFound(vtx_keyword, "s"))?;
        let (tex_v_idx, _) = el
            .scalar_property(&["t", "v", "tex_v"])
            .ok_or(MeshError::PropertyNotFound(vtx_keyword, "t"))?;
        let (norm_x_idx, _) = el
            .scalar_property(&["nx", "norm_x"])
            .ok_or(MeshError::PropertyNotFound(vtx_keyword, "nx"))?;
        let (norm_y_idx, _) = el
            .scalar_property(&["ny", "norm_y"])
            .ok_or(MeshError::PropertyNotFound(vtx_keyword, "ny"))?;
        let (norm_z_idx, _) = el
            .scalar_property(&["nz", "norm_z"])
            .ok_or(MeshError::PropertyNotFound(vtx_keyword, "nz"))?;

        let vertices = data.generate(eidx, |props| {
            let p = [
                props[pos_x_idx].coerce().unwrap(),
                props[pos_y_idx].coerce().unwrap(),
                props[pos_z_idx].coerce().unwrap(),
            ];
            let t = [props[tex_u_idx].coerce().unwrap(), props[tex_v_idx].coerce().unwrap()];
            let n = [
                props[norm_x_idx].coerce().unwrap(),
                props[norm_y_idx].coerce().unwrap(),
                props[norm_z_idx].coerce().unwrap(),
            ];
            Vertex::new(p, t, n)
        });

        let fc_keyword = "face";
        let (eidx, el) = data
            .element(&[fc_keyword, "faces"])
            .ok_or(MeshError::ElementNotFound(fc_keyword))?;
        let (idx, _) = el
            .vector_property(&["vertex_index", "vertex_indices"])
            .ok_or(MeshError::PropertyNotFound(fc_keyword, "vertex_index"))?;

        let indices = data
            .generate(eidx, |p| CoerceTo::<Vec<u16>>::coerce(&p[idx]).unwrap())
            .into_iter()
            .flatten()
            .collect();

        Ok(Mesh { vertices, indices })
    }
}

#[derive(Debug, Fail)]
pub enum MeshError {
    #[fail(display = "The element '{}' was not found", _0)]
    ElementNotFound(&'static str),
    #[fail(display = "The property '{}' was not found on element '{}'", _1, _0)]
    PropertyNotFound(&'static str, &'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path() {
        let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply");

        let r: Result<Mesh, Error> = Mesh::from_path(&p);
        assert_ok!(r);
    }
}
