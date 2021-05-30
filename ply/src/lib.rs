//! The `ply` crate provides facilities to read from Stanford PLY 3d-data files. Supported formats
//! are `ascii`, `big-endian` and `little-endian`, for the specification `1.0`.
#![deny(missing_docs)]

mod macros;
mod parsers;
pub mod types;

pub use self::types::Ply;
use self::types::{Element, PropertyData};
use combine::{
    parser::Parser,
    stream::{buffered::BufferedStream, state::State, ReadStream},
};
use parsers::ply;
use std::{fs::File, io, path::Path};
use thiserror::Error;

/// Describes errors that may occur when parsing a ply file.
#[derive(Debug, Error)]
pub enum Error {
    /// A ply file contains duplicate element names.
    #[error("Some element names appeared more than once")]
    DuplicateElements,
    /// An element in a ply file contains duplicate property names.
    #[error("Some property names appeared more than once in a single element")]
    DuplicateProperties,
    /// General catch-all for `combine` parser errors.
    #[error("{1}: {0}")]
    ParserError(String, String),
    /// General catch-all for IO errors.
    #[error("{1}: {0}")]
    IoError(String, #[source] io::Error),
}

impl Ply {
    /// Loads a ply file from the specified and attempts to parse it. If parsing is successful,
    /// `from_path` also checks for duplicate element or property names. This is necessary, because
    /// otherwise you cannot reliably search for elements or properties by name.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| Error::IoError(format!("{}", path.display()), e))?;

        let stream = BufferedStream::new(State::new(ReadStream::new(file)), 32);
        let data = ply()
            .parse(stream)
            .map(|(d, _)| d)
            .map_err(|e| Error::ParserError(format!("{}", path.display()), format!("{}", e)))?;

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

    /// Returns the last element that matches any of the specified names. Also returns the index of
    /// the element which can be used to obtain the corresponding data from the body of the ply
    /// file.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate ply;
    ///
    /// use std::path::PathBuf;
    /// use ply::Ply;
    ///
    /// let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply"));
    /// let data = Ply::from_path(&path).unwrap();
    ///
    /// assert!(data.element(&["vertex", "vertices"]).is_some());
    /// ```
    pub fn element(&self, names: &[&str]) -> Option<(usize, &Element)> {
        self.header.element(names)
    }

    /// Given an element index and a mapper closure, calls the mapper for each instance of the
    /// supplied element. This allows to map the ply data to other representations.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate ply;
    ///
    /// use std::path::PathBuf;
    /// use ply::Ply;
    /// use std::convert::TryInto;
    ///
    /// let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply"));
    /// let data = Ply::from_path(&path).unwrap();
    ///
    /// let (eidx, el) = data.element(&["face", "faces"]).unwrap();
    /// let (idx, _) = el.vector_property(&["vertex_index", "vertex_indices"]).unwrap();
    ///
    /// let indices: Vec<u16> = data
    ///     .generate(eidx, |p| TryInto::<Vec<u16>>::try_into(&p[idx]).unwrap())
    ///     .into_iter()
    ///     .flatten()
    ///     .collect();
    ///
    /// assert_eq!(indices.len(), 12 * 3);
    /// ```
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
    use std::{convert::TryInto, path::PathBuf};

    #[test]
    fn from_valid_path() {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply"));
        let r = Ply::from_path(&path);

        assert!(r.is_ok());
    }

    #[test]
    fn from_invalid_path() {
        let path = PathBuf::from("/any/invalid/file.ply");
        let r = Ply::from_path(&path);

        assert!(r.is_err());
    }

    #[test]
    fn generate_vertices() {
        #[allow(dead_code)]
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

        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply"));
        let data = Ply::from_path(&path).unwrap();

        let (eidx, el) = data.element(&["vertex", "vertices"]).unwrap();
        let (pos_x_idx, _) = el.scalar_property(&["x", "pos_x"]).unwrap();
        let (pos_y_idx, _) = el.scalar_property(&["y", "pos_y"]).unwrap();
        let (pos_z_idx, _) = el.scalar_property(&["z", "pos_z"]).unwrap();
        let (tex_u_idx, _) = el.scalar_property(&["s", "u", "tex_u"]).unwrap();
        let (tex_v_idx, _) = el.scalar_property(&["t", "v", "tex_v"]).unwrap();
        let (norm_x_idx, _) = el.scalar_property(&["nx", "norm_x"]).unwrap();
        let (norm_y_idx, _) = el.scalar_property(&["ny", "norm_y"]).unwrap();
        let (norm_z_idx, _) = el.scalar_property(&["nz", "norm_z"]).unwrap();

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

        assert_eq!(vertices.len(), 24);
    }

    #[test]
    fn generate_indices() {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply"));
        let data = Ply::from_path(&path).unwrap();

        let (eidx, el) = data.element(&["face", "faces"]).unwrap();
        let (idx_idx, _) = el.vector_property(&["vertex_indices", "vertex_index"]).unwrap();

        let faces = data.generate(eidx, |props| {
            let f: Vec<u16> = (&props[idx_idx]).try_into().unwrap();
            f
        });

        let indices: Vec<u16> = faces.iter().flatten().cloned().collect();

        assert_eq!(faces.len(), 12);
        assert_eq!(indices.len(), 12 * 3);
    }
}
