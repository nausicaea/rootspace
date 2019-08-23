#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mode(u32);

impl Mode {
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

impl PartialEq<u32> for Mode {
    fn eq(&self, rhs: &u32) -> bool {
        self.0 == *rhs
    }
}

impl From<u32> for Mode {
    fn from(value: u32) -> Self {
        Mode(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn user_read(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.user_read(), (mode & 0o400) > 0);
        }

        #[test]
        fn user_write(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.user_write(), (mode & 0o200) > 0);
        }

        #[test]
        fn user_execute(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.user_execute(), (mode & 0o100) > 0);
        }

        #[test]
        fn group_read(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.group_read(), (mode & 0o40) > 0);
        }

        #[test]
        fn group_write(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.group_write(), (mode & 0o20) > 0);
        }

        #[test]
        fn group_execute(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.group_execute(), (mode & 0o10) > 0);
        }

        #[test]
        fn other_read(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.other_read(), (mode & 0o4) > 0);
        }

        #[test]
        fn other_write(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.other_write(), (mode & 0o2) > 0);
        }

        #[test]
        fn other_execute(mode in 0u32..0o777) {
            let m = Mode(mode);
            prop_assert_eq!(m.other_execute(), (mode & 0o1) > 0);
        }
    }
}
