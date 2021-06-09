use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(u32);

impl GroupId {
    pub fn privileged() -> Self {
        GroupId(0)
    }

    pub fn is_privileged(&self) -> bool {
        self.0 == 0
    }
}

impl std::fmt::Display for GroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<u32> for GroupId {
    fn eq(&self, rhs: &u32) -> bool {
        self.0 == *rhs
    }
}

impl From<u32> for GroupId {
    fn from(value: u32) -> Self {
        GroupId(value)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn privileged() {
        let g = GroupId::privileged();
        assert!(g.is_privileged());
    }

    proptest! {
        #[test]
        fn unprivileged(gid in 1u32..65535) {
            let g = GroupId::from(gid);
            prop_assert!(!g.is_privileged());
        }
    }
}
