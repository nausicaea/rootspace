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
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn user_read(mode: u32) -> bool {
        let expected = (mode & 0o400) > 0;
        let m = Mode(mode);
        m.user_read() == expected
    }

    #[quickcheck]
    fn user_write(mode: u32) -> bool {
        let expected = (mode & 0o200) > 0;
        let m = Mode(mode);
        m.user_write() == expected
    }

    #[quickcheck]
    fn user_execute(mode: u32) -> bool {
        let expected = (mode & 0o100) > 0;
        let m = Mode(mode);
        m.user_execute() == expected
    }

    #[quickcheck]
    fn group_read(mode: u32) -> bool {
        let expected = (mode & 0o40) > 0;
        let m = Mode(mode);
        m.group_read() == expected
    }

    #[quickcheck]
    fn group_write(mode: u32) -> bool {
        let expected = (mode & 0o20) > 0;
        let m = Mode(mode);
        m.group_write() == expected
    }

    #[quickcheck]
    fn group_execute(mode: u32) -> bool {
        let expected = (mode & 0o10) > 0;
        let m = Mode(mode);
        m.group_execute() == expected
    }

    #[quickcheck]
    fn other_read(mode: u32) -> bool {
        let expected = (mode & 0o4) > 0;
        let m = Mode(mode);
        m.other_read() == expected
    }

    #[quickcheck]
    fn other_write(mode: u32) -> bool {
        let expected = (mode & 0o2) > 0;
        let m = Mode(mode);
        m.other_write() == expected
    }

    #[quickcheck]
    fn other_execute(mode: u32) -> bool {
        let expected = (mode & 0o1) > 0;
        let m = Mode(mode);
        m.other_execute() == expected
    }
}
