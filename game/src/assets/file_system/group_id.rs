#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupId(u32);

impl GroupId {
    pub fn privileged(&self) -> bool {
        self.0 == 0
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
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn privileged() {
        let g = GroupId::from(0);
        assert!(g.privileged());
    }

    proptest! {
        #[test]
        fn unprivileged(gid in 1u32..65535) {
            let g = GroupId::from(gid);
            prop_assert!(!g.privileged());
        }
    }
}
