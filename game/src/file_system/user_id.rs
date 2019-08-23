#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(u32);

impl UserId {
    pub fn privileged(&self) -> bool {
        self.0 == 0
    }
}

impl PartialEq<u32> for UserId {
    fn eq(&self, rhs: &u32) -> bool {
        self.0 == *rhs
    }
}

impl From<u32> for UserId {
    fn from(value: u32) -> Self {
        UserId(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn privileged(uid: u32) -> bool {
        let expected = uid == 0;
        let u = UserId::from(uid);
        u.privileged() == expected
    }
}
