#[cfg(test)]
#[macro_use]
extern crate assertions;
extern crate combine;
#[macro_use]
extern crate failure;
extern crate log;
extern crate num_traits;

mod parsers;
pub mod types;

pub use self::types::{Ply, PropertyData};
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
}
