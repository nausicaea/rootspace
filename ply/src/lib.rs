#[cfg(test)]
#[macro_use]
extern crate assertions;
extern crate combine;
#[macro_use]
extern crate failure;
extern crate log;
extern crate num_traits;

pub mod parsers;
pub mod types;

pub use self::types::Ply;
use self::types::{Element, Header};
use parsers::ply;
use combine::parser::Parser;
use combine::stream::{ReadStream, buffered::BufferedStream, state::State};
use std::io;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

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

impl Element {
    fn has_duplicate_properties(&self) -> bool {
        let mut unique = HashSet::new();
        !self.properties
            .iter()
            .all(|p| unique.insert(p.name.clone()))
    }
}

impl Header {
    fn has_duplicate_elements(&self) -> bool {
        let mut unique = HashSet::new();
        !self.elements
            .iter()
            .all(|e| unique.insert(e.name.clone()))
    }
}

impl Ply {
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(Error::IoError(format!("{}", path.display()), e)),
        };

        let stream = BufferedStream::new(State::new(ReadStream::new(file)), 32);
        let data = match ply().parse(stream) {
            Ok((data, _)) => data,
            Err(e) => return Err(Error::ParserError(format!("{}", path.display()), format!("{}", e))),
        };

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

    pub fn has_element(&self, name: &str) -> bool {
        self.header.elements
            .iter()
            .any(|e| e.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{Format, FormatType, Property, DataType};
    use std::path::PathBuf;

    #[test]
    fn duplicate_properties() {
        let e = Element {
            name: "vertex".into(),
            count: 1,
            properties: vec![
                Property {
                    name: "x".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
                Property {
                    name: "x".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
            ],
        };

        assert!(e.has_duplicate_properties());
    }

    #[test]
    fn nonduplicate_properties() {
        let e = Element {
            name: "vertex".into(),
            count: 1,
            properties: vec![
                Property {
                    name: "x".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
                Property {
                    name: "y".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                },
            ],
        };

        assert!(!e.has_duplicate_properties());
    }

    #[test]
    fn duplicate_elements() {
        let h = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: vec![
                Element {
                    name: "vertex".into(),
                    count: 0,
                    properties: Vec::new(),
                },
                Element {
                    name: "vertex".into(),
                    count: 0,
                    properties: Vec::new(),
                },
            ],
        };

        assert!(h.has_duplicate_elements());
    }

    #[test]
    fn nonduplicate_elements() {
        let h = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: vec![
                Element {
                    name: "vertex".into(),
                    count: 0,
                    properties: Vec::new(),
                },
                Element {
                    name: "face".into(),
                    count: 0,
                    properties: Vec::new(),
                },
            ],
        };

        assert!(!h.has_duplicate_elements());
    }

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
    fn has_element_accessor() {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube-ascii.ply"));
        let data = Ply::from_path(&path).unwrap();

        assert!(data.has_element("vertex"));
        assert!(data.has_element("face"));
        assert!(!data.has_element("blabla"));
    }
}
