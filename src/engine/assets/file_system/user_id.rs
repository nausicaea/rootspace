use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(u32);

impl UserId {
    pub fn privileged() -> Self {
        UserId(0)
    }

    pub fn is_privileged(&self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn privileged() {
        let u = UserId::privileged();
        assert!(u.is_privileged());
    }

    proptest! {
        #[test]
        fn unprivileged(uid in 1u32..65535) {
            let u = UserId::from(uid);
            prop_assert!(!u.is_privileged());
        }
    }
}
