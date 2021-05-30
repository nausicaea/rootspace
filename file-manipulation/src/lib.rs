#[cfg(any(test, feature = "serde_support"))]
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Read},
    ops::Deref,
    path::{Path, PathBuf},
};
use thiserror::Error;

fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Result<PathBuf, FileError> {
    let p = path_user_input.as_ref();
    if !p.starts_with("~") {
        return Ok(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs::home_dir().ok_or(FileError::NoHomeDirectoryFound);
    }
    dirs::home_dir()
        .map(|mut h| {
            if h == Path::new("/") {
                // Corner case: `h` root directory;
                // don't prepend extra `/`, just drop the tilde.
                p.strip_prefix("~").unwrap().to_path_buf()
            } else {
                h.push(p.strip_prefix("~/").unwrap());
                h
            }
        })
        .ok_or(FileError::NoHomeDirectoryFound)
}

#[cfg_attr(any(test, feature = "serde_support"), derive(Serialize, Deserialize))]
#[cfg_attr(any(test, feature = "serde_support"), serde(transparent))]
#[derive(Clone, PartialEq)]
pub struct NewOrExFilePathBuf(PathBuf);

impl NewOrExFilePathBuf {
    pub fn path(&self) -> &Path {
        AsRef::<Path>::as_ref(self)
    }
}

impl std::fmt::Debug for NewOrExFilePathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for NewOrExFilePathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<PathBuf> for NewOrExFilePathBuf {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for NewOrExFilePathBuf {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<NewOrExFilePathBuf> for PathBuf {
    fn from(path: NewOrExFilePathBuf) -> Self {
        path.0
    }
}

impl<'a> TryFrom<&'a str> for NewOrExFilePathBuf {
    type Error = FileError;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(&Path::new(path))
    }
}

impl TryFrom<PathBuf> for NewOrExFilePathBuf {
    type Error = FileError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(&path)
    }
}

impl TryFrom<OsString> for NewOrExFilePathBuf {
    type Error = FileError;

    fn try_from(path: OsString) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(path.as_ref())
    }
}

impl TryFrom<&OsStr> for NewOrExFilePathBuf {
    type Error = FileError;

    fn try_from(path: &OsStr) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(path.as_ref())
    }
}

impl TryFrom<&Path> for NewOrExFilePathBuf {
    type Error = FileError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let path = expand_tilde(path)?;

        if !path.exists() {
            let parent = path
                .parent()
                .filter(|p| p.is_dir())
                .ok_or_else(|| FileError::ParentDirectoryNotFound(path.to_path_buf()))
                .and_then(|p| p.canonicalize().map_err(|e| FileError::IoError(path.to_path_buf(), e)))?;

            let file_name = path.file_name().ok_or_else(|| FileError::NoBaseNameFound(path.to_path_buf()))?;

            Ok(NewOrExFilePathBuf(parent.join(file_name)))
        } else if path.is_file() {
            let path = path
                .canonicalize()
                .map_err(|e| FileError::IoError(path.to_path_buf(), e))?;
            Ok(NewOrExFilePathBuf(path))
        } else {
            Err(FileError::NotAFile(path))
        }
    }
}

#[cfg_attr(any(test, feature = "serde_support"), derive(Serialize, Deserialize))]
#[cfg_attr(any(test, feature = "serde_support"), serde(transparent))]
#[derive(Clone, PartialEq)]
pub struct FilePathBuf(PathBuf);

impl FilePathBuf {
    pub fn path(&self) -> &Path {
        AsRef::<Path>::as_ref(self)
    }

    pub fn read_to_string(&self) -> Result<String, FileError> {
        let mut f = File::open(&self.0).map_err(|e| FileError::IoError(self.0.clone(), e))?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .map_err(|e| FileError::IoError(self.0.clone(), e))?;

        Ok(buf)
    }

    pub fn read_to_bytes(&self) -> Result<Vec<u8>, FileError> {
        let mut f = File::open(&self.0).map_err(|e| FileError::IoError(self.0.clone(), e))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .map_err(|e| FileError::IoError(self.0.clone(), e))?;

        Ok(buf)
    }
}

impl std::fmt::Debug for FilePathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for FilePathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<PathBuf> for FilePathBuf {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for FilePathBuf {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<FilePathBuf> for PathBuf {
    fn from(path: FilePathBuf) -> Self {
        path.0
    }
}

impl<'a> TryFrom<&'a str> for FilePathBuf {
    type Error = FileError;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(&Path::new(path))
    }
}

impl TryFrom<PathBuf> for FilePathBuf {
    type Error = FileError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(&path)
    }
}

impl TryFrom<OsString> for FilePathBuf {
    type Error = FileError;

    fn try_from(path: OsString) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(path.as_ref())
    }
}

impl TryFrom<&OsStr> for FilePathBuf {
    type Error = FileError;

    fn try_from(path: &OsStr) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(path.as_ref())
    }
}

impl<'a> TryFrom<&Path> for FilePathBuf {
    type Error = FileError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let path = expand_tilde(path)?;

        if path.is_file() {
            let path = path
                .canonicalize()
                .map_err(|e| FileError::IoError(path.to_path_buf(), e))?;
            Ok(FilePathBuf(path))
        } else {
            Err(FileError::NotAFile(path))
        }
    }
}

#[cfg_attr(any(test, feature = "serde_support"), derive(Serialize, Deserialize))]
#[cfg_attr(any(test, feature = "serde_support"), serde(transparent))]
#[derive(Clone, PartialEq)]
pub struct DirPathBuf(PathBuf);

impl DirPathBuf {
    pub fn path(&self) -> &Path {
        AsRef::<Path>::as_ref(self)
    }
}

impl std::fmt::Debug for DirPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for DirPathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<PathBuf> for DirPathBuf {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for DirPathBuf {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<DirPathBuf> for PathBuf {
    fn from(path: DirPathBuf) -> Self {
        path.0
    }
}

impl<'a> TryFrom<&'a str> for DirPathBuf {
    type Error = FileError;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(&Path::new(path))
    }
}

impl TryFrom<PathBuf> for DirPathBuf {
    type Error = FileError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(&path)
    }
}

impl TryFrom<OsString> for DirPathBuf {
    type Error = FileError;

    fn try_from(path: OsString) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(path.as_ref())
    }
}

impl TryFrom<&OsStr> for DirPathBuf {
    type Error = FileError;

    fn try_from(path: &OsStr) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(path.as_ref())
    }
}

impl<'a> TryFrom<&Path> for DirPathBuf {
    type Error = FileError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let path = expand_tilde(path)?;

        if path.is_dir() {
            let path = path
                .canonicalize()
                .map_err(|e| FileError::IoError(path.to_path_buf(), e))?;
            Ok(DirPathBuf(path))
        } else {
            Err(FileError::NotADirectory(path))
        }
    }
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("No such file or directory: {}", .0.display())]
    FileOrDirectoryNotFound(PathBuf),
    #[error("Not a file: {}", .0.display())]
    NotAFile(PathBuf),
    #[error("Not a directory: {}", .0.display())]
    NotADirectory(PathBuf),
    #[error("Parent directory not found: {}", .0.display())]
    ParentDirectoryNotFound(PathBuf),
    #[error("{}: {}", .1, .0.display())]
    IoError(PathBuf, #[source] io::Error),
    #[error("Path does not contain a basename: {}", .0.display())]
    NoBaseNameFound(PathBuf),
    #[error("Could not find the user home directory")]
    NoHomeDirectoryFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    #[cfg_attr(not(target_family = "unix"), ignore)]
    fn test_expand_tilde() {
        // Should work on your linux box during tests, would fail in stranger
        // environments!
        let home = std::env::var("HOME").unwrap();
        let projects = PathBuf::from(format!("{}/Projects", home));
        assert_eq!(expand_tilde("~/Projects").unwrap(), projects);
        assert_eq!(expand_tilde("/foo/bar").unwrap(), Path::new("/foo/bar"));
        assert_eq!(expand_tilde("~alice/projects").unwrap(), Path::new("~alice/projects"));
    }

    #[test]
    fn new_or_ex_file_path() {
        let tf = NamedTempFile::new().unwrap();
        let base_dir = tempdir().unwrap();

        // The operation must succeed for an existing file
        let r = NewOrExFilePathBuf::try_from(tf.path());
        assert!(r.is_ok(), "{:?}", r.unwrap_err());

        // The operation must succeed for a path whose basename does not exist
        let new_file = base_dir.path().join("newfile.txt");
        let r = NewOrExFilePathBuf::try_from(new_file);
        assert!(r.is_ok(), "{:?}", r.unwrap_err());

        // The operation must fail for a path whose parent does not exist
        let bad_new_file = base_dir.path().join("/i/do/not/exist.tmp");
        let r = NewOrExFilePathBuf::try_from(bad_new_file);
        assert!(r.is_err());

        // The operation must fail for a directory
        let r = NewOrExFilePathBuf::try_from(base_dir.path());
        assert!(r.is_err())
    }

    #[test]
    fn file_path() {
        let tf = NamedTempFile::new().unwrap();
        let base_dir = tempdir().unwrap();

        // The operation must succeed for an existing file
        let r = FilePathBuf::try_from(tf.path());
        assert!(r.is_ok(), "{:?}", r.unwrap_err());

        // The operation must fail for a path that does not exist
        let new_file = base_dir.path().join("newfile.txt");
        let r = FilePathBuf::try_from(new_file);
        assert!(r.is_err());

        // The operation must fail for a path whose parent does not exist
        let bad_new_file = base_dir.path().join("/i/do/not/exist.tmp");
        let r = FilePathBuf::try_from(bad_new_file);
        assert!(r.is_err());

        // The operation must fail for a directory
        let r = FilePathBuf::try_from(base_dir.path());
        assert!(r.is_err())
    }

    #[test]
    fn directory_path() {
        let tf = NamedTempFile::new().unwrap();
        let base_dir = tempdir().unwrap();

        // The operation must fail for an existing file
        let r = DirPathBuf::try_from(tf.path());
        assert!(r.is_err());

        // The operation must fail for a path that does not exist
        let new_file = base_dir.path().join("newdir");
        let r = DirPathBuf::try_from(new_file);
        assert!(r.is_err());

        // The operation must fail for a path whose parent does not exist
        let bad_new_file = base_dir.path().join("/i/do/not/exist");
        let r = DirPathBuf::try_from(bad_new_file);
        assert!(r.is_err());

        // The operation must succeed for a directory
        let r = DirPathBuf::try_from(base_dir.path());
        assert!(r.is_ok(), "{:?}", r.unwrap_err())
    }

    #[test]
    fn file_path_read_to_string() {
        let mut tf = NamedTempFile::new().unwrap();

        write!(tf, "Hello, World!").unwrap();

        let r = FilePathBuf::try_from(tf.path()).unwrap().read_to_string();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), "Hello, World!");
    }

    #[test]
    fn file_path_read_to_bytes() {
        let mut tf = NamedTempFile::new().unwrap();

        tf.write(&[0x00, 0xff, 0x14, 0xf6]).unwrap();

        let r = FilePathBuf::try_from(tf.path()).unwrap().read_to_bytes();
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), vec![0x00, 0xff, 0x14, 0xf6]);
    }

    #[test]
    fn new_file_path_serde() {
        let fstr = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/nonex-file");
        let nfp = NewOrExFilePathBuf::try_from(fstr).unwrap();

        assert_tokens(&nfp, &[Token::Str(fstr)]);

        let fstr = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/empty-file");
        let nfp = NewOrExFilePathBuf::try_from(fstr).unwrap();

        assert_tokens(&nfp, &[Token::Str(fstr)]);
    }

    #[test]
    fn file_path_serde() {
        let fstr = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/empty-file");
        let fp = FilePathBuf::try_from(fstr).unwrap();

        assert_tokens(&fp, &[Token::Str(fstr)]);
    }

    #[test]
    fn dir_path_serde() {
        let dstr = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");
        let dp = DirPathBuf::try_from(dstr).unwrap();

        assert_tokens(&dp, &[Token::Str(dstr)]);
    }
}
