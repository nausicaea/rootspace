use image;
use std::{
    borrow::Borrow,
    fmt,
    fs::File,
    io::{self, Read},
    path::Path,
};

pub struct Image(image::DynamicImage);

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Image(image::DynamicImage)")
    }
}

#[derive(Debug)]
pub struct Mesh;

pub trait VerifyPath {
    /// Verifies that `self` refers to an existing file in the file system.
    fn ensure_extant_file(&self) -> Result<(), FileError>;
    /// Verifies that `self` refers to an existing directory in the file system.
    fn ensure_extant_directory(&self) -> Result<(), FileError>;
}

impl<T: Borrow<Path>> VerifyPath for T {
    fn ensure_extant_file(&self) -> Result<(), FileError> {
        let path = self.borrow();
        if path.exists() {
            if path.is_file() {
                Ok(())
            } else {
                Err(FileError::NotAFile(format!("{}", path.display())))
            }
        } else {
            Err(FileError::FileOrDirectoryNotFound(format!("{}", path.display())))
        }
    }

    fn ensure_extant_directory(&self) -> Result<(), FileError> {
        let path = self.borrow();
        if path.exists() {
            if path.is_dir() {
                Ok(())
            } else {
                Err(FileError::NotADirectory(format!("{}", path.display())))
            }
        } else {
            Err(FileError::FileOrDirectoryNotFound(format!("{}", path.display())))
        }
    }
}

pub trait ReadPath {
    /// Reads `self` as a string of UTF-8 characters.
    fn read_to_string(&self) -> Result<String, FileError>;
    /// Reads `self` as a vector of bytes.
    fn read_to_bytes(&self) -> Result<Vec<u8>, FileError>;
    /// Reads `self` as an image.
    fn read_to_image(&self) -> Result<Image, FileError>;
    /// Reads `self` as 3-dimensional vertex data.
    fn read_to_mesh(&self) -> Result<Mesh, FileError>;
}

impl<T: Borrow<Path>> ReadPath for T {
    fn read_to_string(&self) -> Result<String, FileError> {
        let path = self.borrow();
        path.ensure_extant_file()?;
        let mut f = File::open(path).map_err(|e| FileError::IoError(format!("{}", path.display()), e))?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .map_err(|e| FileError::IoError(format!("{}", path.display()), e))?;

        Ok(buf)
    }

    fn read_to_bytes(&self) -> Result<Vec<u8>, FileError> {
        let path = self.borrow();
        path.ensure_extant_file()?;
        let mut f = File::open(path).map_err(|e| FileError::IoError(format!("{}", path.display()), e))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .map_err(|e| FileError::IoError(format!("{}", path.display()), e))?;

        Ok(buf)
    }

    fn read_to_image(&self) -> Result<Image, FileError> {
        let path = self.borrow();
        path.ensure_extant_file()?;
        image::open(path)
            .map(|img| Image(img))
            .map_err(|e| FileError::ImageError(format!("{}", path.display()), e))
    }

    fn read_to_mesh(&self) -> Result<Mesh, FileError> {
        let path = self.borrow();
        path.ensure_extant_file()?;
        unimplemented!()
    }
}

#[derive(Debug, Fail)]
pub enum FileError {
    #[fail(display = "No such file or directory: {}", _0)]
    FileOrDirectoryNotFound(String),
    #[fail(display = "Not a file: {}", _0)]
    NotAFile(String),
    #[fail(display = "Not a directory: {}", _0)]
    NotADirectory(String),
    #[fail(display = "{}: {}", _1, _0)]
    ImageError(String, #[cause] image::ImageError),
    #[fail(display = "{}: {}", _1, _0)]
    IoError(String, #[cause] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, path::PathBuf};
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn ensure_extant_file() {
        let tf = NamedTempFile::new().unwrap();
        let base_dir = tempdir().unwrap();

        let r = tf.path().ensure_extant_file();
        assert_ok!(r);

        let bad_file = base_dir.path().join("blabla.ext");
        let r = bad_file.ensure_extant_file();
        assert_err!(r);

        let bad_dir = base_dir.path();
        let r = bad_dir.ensure_extant_file();
        assert_err!(r);
    }

    #[test]
    fn ensure_extant_directory() {
        let tf = NamedTempFile::new().unwrap();
        let base_dir = tempdir().unwrap();

        let r = tf.path().ensure_extant_directory();
        assert_err!(r);

        let bad_dir = base_dir.path().join("blabla");
        let r = bad_dir.ensure_extant_directory();
        assert_err!(r);

        let good_dir = base_dir.path();
        let r = good_dir.ensure_extant_directory();
        assert_ok!(r);
    }

    #[test]
    fn read_to_string() {
        let mut tf = NamedTempFile::new().unwrap();

        write!(tf, "Hello, World!");

        let r = tf.path().read_to_string();
        assert_ok!(r);
        assert_eq!(r.unwrap(), "Hello, World!");
    }

    #[test]
    fn read_to_bytes() {
        let mut tf = NamedTempFile::new().unwrap();

        tf.write(&[0x00, 0xff, 0x14, 0xf6]).unwrap();

        let r = tf.path().read_to_bytes();
        assert_ok!(r);
        assert_eq!(r.unwrap(), vec![0x00, 0xff, 0x14, 0xf6]);
    }

    #[test]
    fn read_to_image() {
        let img_file = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/tv-test-image.png"));
        let r: Result<Image, FileError> = img_file.read_to_image();
        assert_ok!(r);
    }

    #[test]
    fn read_to_mesh() {
        let mesh_file = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply"));
        let r: Result<Mesh, FileError> = mesh_file.read_to_mesh();
        assert_ok!(r);
    }
}
