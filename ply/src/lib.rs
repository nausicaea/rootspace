#[cfg(test)]
#[macro_use]
extern crate assertions;
extern crate combine;
#[macro_use]
extern crate failure;
extern crate log;
extern crate num_traits;

#[macro_use]
mod macros;
mod parsers;
pub mod types;

pub use self::types::{Ply, CoerceTo};
use self::types::{PropertyData, Element};
use combine::{
    parser::Parser,
    stream::{buffered::BufferedStream, state::State, ReadStream},
};
use parsers::ply;
use std::{fs::File, io, path::Path};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Some element names appeared more than once")]
    DuplicateElements,
    #[fail(display = "Some property names appeared more than once in a single element")]
    DuplicateProperties,
    #[fail(display = "{}: {}", _1, _0)]
    ParserError(String, String),
    #[fail(display = "{}: {}", _1, _0)]
    IoError(String, #[cause] io::Error),
}

impl Ply {
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        let file = File::open(path)
            .map_err(|e| Error::IoError(format!("{}", path.display()), e))?;

        let stream = BufferedStream::new(State::new(ReadStream::new(file)), 32);
        let data = ply().parse(stream)
            .map(|(d, _)| d)
            .map_err( |e|Error::ParserError(format!("{}", path.display()), format!("{}", e)))?;

        if !data.header.has_duplicate_elements() {
            if !data.header.elements.iter().any(|e| e.has_duplicate_properties()) {
                Ok(data)
            } else {
                Err(Error::DuplicateProperties)
            }
        } else {
            Err(Error::DuplicateElements)
        }
    }

    pub fn element(&self, names: &[&str]) -> Option<(usize, &Element)> {
        self.header.element(names)
    }

    pub fn generate<T, F>(&self, element: usize, mapper: F) -> Vec<T>
    where
        F: Fn(&[PropertyData]) -> T,
    {
        self.body.generate(element, mapper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn from_valid_path() {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube-ascii.ply"));
        let r = Ply::from_path(&path);

        assert_ok!(r);
    }

    #[test]
    fn from_invalid_path() {
        let path = PathBuf::from("/any/invalid/file.ply");
        let r = Ply::from_path(&path);

        assert_err!(r);
    }

    #[test]
    fn generate_vertices() {
        struct Vertex {
            position: [f32; 3],
            tex_coord: [f32; 2],
            normal: [f32; 3],
        }

        impl Vertex {
            fn new(p: [f32; 3], t: [f32; 2], n: [f32; 3]) -> Self {
                Vertex {
                    position: p,
                    tex_coord: t,
                    normal: n,
                }
            }
        }

        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube-ascii.ply"));
        let data = Ply::from_path(&path).unwrap();

        let (eidx, el) = data.element(&["vertex", "vertices"]).unwrap();
        let (pos_x_idx, _) = el.scalar_property(&["x", "pos_x"]).unwrap();
        let (pos_y_idx, _) = el.scalar_property(&["y", "pos_y"]).unwrap();
        let (pos_z_idx, _) = el.scalar_property(&["z", "pos_z"]).unwrap();
        let (tex_u_idx, _) = el.scalar_property(&["s", "u", "tex_u"]).unwrap();
        let (tex_v_idx, _) = el.scalar_property(&["t", "v", "tex_v"]).unwrap();
        let (norm_x_idx, _) = el.scalar_property(&["norm_x"]).unwrap();
        let (norm_y_idx, _) = el.scalar_property(&["norm_y"]).unwrap();
        let (norm_z_idx, _) = el.scalar_property(&["norm_z"]).unwrap();

        let vertices = data.generate(eidx, |props| {
            let p = [
                props[pos_x_idx].coerce().unwrap(),
                props[pos_y_idx].coerce().unwrap(),
                props[pos_z_idx].coerce().unwrap(),
            ];
            let t = [
                props[tex_u_idx].coerce().unwrap(),
                props[tex_v_idx].coerce().unwrap(),
            ];
            let n = [
                props[norm_x_idx].coerce().unwrap(),
                props[norm_y_idx].coerce().unwrap(),
                props[norm_z_idx].coerce().unwrap(),
            ];
            Vertex::new(p, t, n)
        });

        assert_eq!(vertices.len(), 24);
    }

    #[test]
    fn generate_indices() {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube-ascii.ply"));
        let data = Ply::from_path(&path).unwrap();

        let (eidx, el) = data.element(&["face", "faces"]).unwrap();
        let (idx_idx, _) = el.vector_property(&["vertex_index", "vertex_indices"]).unwrap();

        let faces = data.generate(eidx, |props| {
            let f: Vec<u16> = props[idx_idx].coerce().unwrap();
            f
        });

        let indices: Vec<u16> = faces
            .iter()
            .flatten()
            .cloned()
            .collect();

        assert_eq!(faces.len(), 12);
        assert_eq!(indices.len(), 12 * 3);
    }
}
