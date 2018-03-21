use std::path::PathBuf;

pub trait VerifyPath {
    fn ensure_accessible_file(self) -> Result<Self, FileError> where Self: Sized;
    fn ensure_accessible_directory(self) -> Result<Self, FileError> where Self: Sized;
}

impl VerifyPath for PathBuf {
    fn ensure_accessible_file(self) -> Result<Self, FileError> where Self: Sized {
        if self.exists() {
            if self.is_file() {
                Ok(self)
            } else {
                Err(FileError::NotAFile(format!("{}", self.display())))
            }
        } else {
            Err(FileError::FileOrDirectoryNotFound(format!("{}", self.display())))
        }
    }
    fn ensure_accessible_directory(self) -> Result<Self, FileError> where Self: Sized {
        if self.exists() {
            if self.is_dir() {
                Ok(self)
            } else {
                Err(FileError::NotADirectory(format!("{}", self.display())))
            }
        } else {
            Err(FileError::FileOrDirectoryNotFound(format!("{}", self.display())))
        }
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
}

#[cfg(test)]
mod tests {
    use std::env;
    use tempfile::NamedTempFileOptions;
    use super::*;

    #[test]
    fn ensure_accessible_file_for_path_buf() {
        let tf = NamedTempFileOptions::new()
            .create()
            .unwrap();

        let tf_path = tf.path().to_path_buf();
        let r = tf_path.ensure_accessible_file();
        assert!(r.is_ok(), "Expected a path, but got the error '{}' instead", r.unwrap_err());

        let r = r.unwrap();
        assert_eq!(r, tf.path(), "Expected the path '{}', but got '{}' instead", tf.path().display(), r.display());

        let bad_file = env::temp_dir().join("blabla.ext");
        let r = bad_file.ensure_accessible_file();
        assert!(r.is_err(), "Expected an error, but got the path '{}' instead", r.unwrap().display());

        let bad_dir = env::temp_dir().join(".");
        let r = bad_dir.ensure_accessible_file();
        assert!(r.is_err(), "Expected an error, but got the path '{}' instead", r.unwrap().display());
    }
    #[test]
    fn ensure_accessible_directory_for_path_buf() {
        let good_dir = env::temp_dir();
        let r = good_dir.ensure_accessible_directory();
        assert!(r.is_ok(), "Expected a path, but got the error '{}' instead", r.unwrap_err());

        let r = r.unwrap();
        assert_eq!(r, env::temp_dir(), "Expected the path '{}' but got '{}' instead", env::temp_dir().display(), r.display());

        let bad_dir = env::temp_dir().join("blabla");
        let r = bad_dir.ensure_accessible_directory();
        assert!(r.is_err(), "Expected an error, but got the path '{}' instead", r.unwrap().display());

        let tf = NamedTempFileOptions::new()
            .create()
            .unwrap();

        let tf_path = tf.path().to_path_buf();
        let r = tf_path.ensure_accessible_directory();
        assert!(r.is_err(), "Expected an error, but got the path '{}' instead", r.unwrap().display());
    }
}
