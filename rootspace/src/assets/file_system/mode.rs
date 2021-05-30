use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use thiserror::Error;

static FILE_TYPE_MASK: u16 = 0o170000;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The value {0:06o} does not correspond to a known file type")]
    UnknownFileType(u16),
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Mode(u16);

impl Mode {
    pub fn builder() -> ModeBuilder {
        ModeBuilder::default()
    }

    pub fn directory(mask: Mode) -> Self {
        ModeBuilder::default()
            .with_mask(mask)
            .with_type(FileType::Directory)
            .with_user_perms(Permission::ALL)
            .with_group_perms(Permission::ALL)
            .with_other_perms(Permission::ALL)
            .build()
    }

    pub fn file(mask: Mode) -> Self {
        ModeBuilder::default()
            .with_mask(mask)
            .with_type(FileType::RegularFile)
            .with_user_perms(Permission::READ | Permission::WRITE)
            .with_group_perms(Permission::READ | Permission::WRITE)
            .with_other_perms(Permission::READ | Permission::WRITE)
            .build()
    }

    pub fn file_type(&self) -> FileType {
        TryFrom::<u16>::try_from(self.0 & FILE_TYPE_MASK).expect("The mode contains an invalid file type value")
    }

    pub fn set_uid(&self) -> bool {
        (self.0 & 0o4000) > 0
    }

    pub fn set_gid(&self) -> bool {
        (self.0 & 0o2000) > 0
    }

    pub fn sticky(&self) -> bool {
        (self.0 & 0o1000) > 0
    }

    pub fn user_read(&self) -> bool {
        (self.0 & 0o400) > 0
    }

    pub fn user_write(&self) -> bool {
        (self.0 & 0o200) > 0
    }

    pub fn user_execute(&self) -> bool {
        (self.0 & 0o100) > 0
    }

    pub fn group_read(&self) -> bool {
        (self.0 & 0o40) > 0
    }

    pub fn group_write(&self) -> bool {
        (self.0 & 0o20) > 0
    }

    pub fn group_execute(&self) -> bool {
        (self.0 & 0o10) > 0
    }

    pub fn other_read(&self) -> bool {
        (self.0 & 0o4) > 0
    }

    pub fn other_write(&self) -> bool {
        (self.0 & 0o2) > 0
    }

    pub fn other_execute(&self) -> bool {
        (self.0 & 0o1) > 0
    }

    pub fn any_execute(&self) -> bool {
        self.user_execute() || self.group_execute() || self.other_execute()
    }
}

impl std::fmt::Debug for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mode({:06o})", self.0)
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:06o}", self.0)
    }
}

impl PartialEq<u16> for Mode {
    fn eq(&self, rhs: &u16) -> bool {
        &self.0 == rhs
    }
}

impl From<u16> for Mode {
    fn from(value: u16) -> Self {
        Mode(value)
    }
}

impl From<Mode> for u16 {
    fn from(value: Mode) -> Self {
        value.0
    }
}

impl From<ModeBuilder> for Mode {
    fn from(value: ModeBuilder) -> Self {
        let mut packed = 0u16;

        // Pack the file type
        packed |= Into::<u16>::into(value.file_type) & FILE_TYPE_MASK;

        // Pack the set-uid, set-gid and sticky bits
        packed |= if value.set_uid { 1 << 11 } else { 0 };
        packed |= if value.set_gid { 1 << 10 } else { 0 };
        packed |= if value.sticky { 1 << 9 } else { 0 };

        // Pack the permission scopes
        packed |= ((value.user_perms.bits as u16) & 0o7) << 6;
        packed |= ((value.group_perms.bits as u16) & 0o7) << 3;
        packed |= (value.other_perms.bits as u16) & 0o7;

        // Apply the permission mask to the packed bits, if one is set
        if let Some(mask) = value.mask {
            packed &= !(mask.0 & 0o777);
        }

        Mode(packed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    FifoPipe,
    CharacterDevice,
    Directory,
    BlockDevice,
    RegularFile,
    Symlink,
    Socket,
}

impl From<FileType> for u16 {
    fn from(value: FileType) -> Self {
        use self::FileType::*;

        match value {
            FifoPipe => 0o010000,
            CharacterDevice => 0o020000,
            Directory => 0o040000,
            BlockDevice => 0o060000,
            RegularFile => 0o100000,
            Symlink => 0o120000,
            Socket => 0o140000,
        }
    }
}

impl TryFrom<u16> for FileType {
    type Error = Error;

    fn try_from(value: u16) -> Result<FileType, Self::Error> {
        use self::FileType::*;

        match value {
            0o010000 => Ok(FifoPipe),
            0o020000 => Ok(CharacterDevice),
            0o040000 => Ok(Directory),
            0o060000 => Ok(BlockDevice),
            0o100000 => Ok(RegularFile),
            0o120000 => Ok(Symlink),
            0o140000 => Ok(Socket),
            _ => Err(Error::UnknownFileType(value)),
        }
    }
}

bitflags! {
    pub struct Permission: u8 {
        const NONE = 0;
        const READ = 4;
        const WRITE = 2;
        const EXECUTE = 1;
        const ALL = Permission::READ.bits | Permission::WRITE.bits | Permission::EXECUTE.bits;
    }
}

#[derive(Debug, Clone)]
pub struct ModeBuilder {
    mask: Option<Mode>,
    file_type: FileType,
    set_uid: bool,
    set_gid: bool,
    sticky: bool,
    user_perms: Permission,
    group_perms: Permission,
    other_perms: Permission,
}

impl ModeBuilder {
    pub fn new() -> Self {
        ModeBuilder::default()
    }

    pub fn with_mask(mut self, mask: Mode) -> Self {
        self.mask = Some(mask);
        self
    }

    pub fn with_type(mut self, ft: FileType) -> Self {
        self.file_type = ft;
        self
    }

    pub fn with_set_uid(mut self, status: bool) -> Self {
        self.set_uid = status;
        self
    }

    pub fn with_set_gid(mut self, status: bool) -> Self {
        self.set_gid = status;
        self
    }

    pub fn with_sticky(mut self, status: bool) -> Self {
        self.sticky = status;
        self
    }

    pub fn with_user_perms(mut self, perms: Permission) -> Self {
        self.user_perms = perms;
        self
    }

    pub fn with_group_perms(mut self, perms: Permission) -> Self {
        self.group_perms = perms;
        self
    }

    pub fn with_other_perms(mut self, perms: Permission) -> Self {
        self.other_perms = perms;
        self
    }

    pub fn build(self) -> Mode {
        self.into()
    }
}

impl Default for ModeBuilder {
    fn default() -> Self {
        ModeBuilder {
            mask: None,
            file_type: FileType::RegularFile,
            set_uid: false,
            set_gid: false,
            sticky: false,
            user_perms: Permission::NONE,
            group_perms: Permission::NONE,
            other_perms: Permission::NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn directory() {
        assert_eq!(Mode::directory(Mode(0o022)), Mode(0o040755));
    }

    #[test]
    fn file() {
        assert_eq!(Mode::file(Mode(0o022)), Mode(0o100644));
    }

    #[test]
    fn sticky() {
        assert!(Mode(0o1000).sticky());
    }

    #[test]
    fn set_gid() {
        assert!(Mode(0o2000).set_gid());
    }

    #[test]
    fn set_uid() {
        assert!(Mode(0o4000).set_uid());
    }

    #[test]
    fn file_type() {
        assert_eq!(Mode(0o010000).file_type(), FileType::FifoPipe);
        assert_eq!(Mode(0o020000).file_type(), FileType::CharacterDevice);
        assert_eq!(Mode(0o040000).file_type(), FileType::Directory);
        assert_eq!(Mode(0o060000).file_type(), FileType::BlockDevice);
        assert_eq!(Mode(0o100000).file_type(), FileType::RegularFile);
        assert_eq!(Mode(0o120000).file_type(), FileType::Symlink);
        assert_eq!(Mode(0o140000).file_type(), FileType::Socket);
    }

    #[test]
    #[should_panic]
    fn unknown_file_type() {
        Mode(0o160000).file_type();
    }

    proptest! {
        #[test]
        fn user_read(mode in 0u16..0o777) {
            prop_assert_eq!(Mode(mode).user_read(), (mode & 0o400) > 0);
        }

        #[test]
        fn user_write(mode in 0u16..0o777) {
            prop_assert_eq!(Mode(mode).user_write(), (mode & 0o200) > 0);
        }

        #[test]
        fn user_execute(mode in 0u16..0o777) {
            prop_assert_eq!(Mode(mode).user_execute(), (mode & 0o100) > 0);
        }

        #[test]
        fn group_read(mode in 0u16..0o777) {
            prop_assert_eq!(Mode(mode).group_read(), (mode & 0o40) > 0);
        }

        #[test]
        fn group_write(mode in 0u16..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.group_write(), (mode & 0o20) > 0);
        }

        #[test]
        fn group_execute(mode in 0u16..0o777) {
            prop_assert_eq!(Mode(mode).group_execute(), (mode & 0o10) > 0);
        }

        #[test]
        fn other_read(mode in 0u16..0o777) {
            prop_assert_eq!(Mode(mode).other_read(), (mode & 0o4) > 0);
        }

        #[test]
        fn other_write(mode in 0u16..0o777) {
            prop_assert_eq!(Mode(mode).other_write(), (mode & 0o2) > 0);
        }

        #[test]
        fn other_execute(mode in 0u16..0o777) {
            prop_assert_eq!(Mode(mode).other_execute(), (mode & 0o1) > 0);
        }
    }
}
