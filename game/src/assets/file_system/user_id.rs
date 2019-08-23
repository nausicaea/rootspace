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
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn privileged(uid in 0u32..65535) {
            let u = UserId::from(uid);
            prop_assert_eq!(u.privileged(), uid == 0);
        }
    }
}
