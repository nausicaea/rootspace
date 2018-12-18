use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

pub trait VerifyPath {
    /// Verifies that `self` refers to an existing file in the file system.
    fn ensure_extant_file(&self) -> Result<(), FileError>;
    /// Verifies that `self` refers to an existing directory in the file system.
    fn ensure_extant_directory(&self) -> Result<(), FileError>;
}

impl<T: AsRef<Path>> VerifyPath for T {
    fn ensure_extant_file(&self) -> Result<(), FileError> {
        let path = self.as_ref();
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
        let path = self.as_ref();
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
}

impl<T: AsRef<Path>> ReadPath for T {
    fn read_to_string(&self) -> Result<String, FileError> {
        let path = self.as_ref();
        path.ensure_extant_file()?;
        let mut f = File::open(path).map_err(|e| FileError::IoError(format!("{}", path.display()), e))?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .map_err(|e| FileError::IoError(format!("{}", path.display()), e))?;

        Ok(buf)
    }

    fn read_to_bytes(&self) -> Result<Vec<u8>, FileError> {
        let path = self.as_ref();
        path.ensure_extant_file()?;
        let mut f = File::open(path).map_err(|e| FileError::IoError(format!("{}", path.display()), e))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .map_err(|e| FileError::IoError(format!("{}", path.display()), e))?;

        Ok(buf)
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
    IoError(String, #[cause] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn ensure_extant_file() {
        let tf = NamedTempFile::new().unwrap();
        let base_dir = tempdir().unwrap();

        let r = tf.path().ensure_extant_file();
        assert!(r.is_ok());

        let bad_file = base_dir.path().join("blabla.ext");
        let r = bad_file.ensure_extant_file();
        assert!(r.is_err());

        let bad_dir = base_dir.path();
        let r = bad_dir.ensure_extant_file();
        assert!(r.is_err());
    }

    #[test]
    fn ensure_extant_directory() {
        let tf = NamedTempFile::new().unwrap();
        let base_dir = tempdir().unwrap();

        let r = tf.path().ensure_extant_directory();
        assert!(r.is_err());

        let bad_dir = base_dir.path().join("blabla");
        let r = bad_dir.ensure_extant_directory();
        assert!(r.is_err());

        let good_dir = base_dir.path();
        let r = good_dir.ensure_extant_directory();
        assert!(r.is_ok());
    }

    #[test]
    fn read_to_string() {
        let mut tf = NamedTempFile::new().unwrap();

        write!(tf, "Hello, World!").unwrap();

        let r = tf.path().read_to_string();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), "Hello, World!");
    }

    #[test]
    fn read_to_bytes() {
        let mut tf = NamedTempFile::new().unwrap();

        tf.write(&[0x00, 0xff, 0x14, 0xf6]).unwrap();

        let r = tf.path().read_to_bytes();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), vec![0x00, 0xff, 0x14, 0xf6]);
    }
}
