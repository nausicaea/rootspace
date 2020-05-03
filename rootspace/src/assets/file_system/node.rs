use super::{group_id::GroupId, mode::Mode, user_id::UserId};
use chrono::{DateTime, Utc};
use std::ffi::{OsStr, OsString};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    name: Option<OsString>,
    uid: UserId,
    gid: GroupId,
    mode: Mode,
    accessed: DateTime<Utc>,
    modified: DateTime<Utc>,
    changed: DateTime<Utc>,
}

impl Node {
    pub fn new<U: Into<UserId>, G: Into<GroupId>, M: Into<Mode>>(
        name: Option<&OsStr>,
        uid: U,
        gid: G,
        mode: M,
    ) -> Self {
        let now = Utc::now();

        Node {
            name: name.map(|n| n.to_owned()),
            uid: uid.into(),
            gid: gid.into(),
            mode: mode.into(),
            accessed: now,
            modified: now,
            changed: now,
        }
    }

    pub fn name(&self) -> Option<&OsStr> {
        self.name.as_deref()
    }

    /// Return `true` if the supplied UID and GIDs have read permission on this node. A privileged
    /// user (UID 0) always has read access.
    pub fn may_read(&self, uid: &UserId, gids: &[GroupId]) -> bool {
        uid.is_privileged()
            || ((self.uid == *uid) && self.mode.user_read())
            || (gids.iter().any(|gid| self.gid == *gid) && self.mode.group_read())
            || self.mode.other_read()
    }

    /// Return `true` if the supplied UID and GIDs have write permission on this node. A privileged
    /// user (UID 0) always has write access.
    pub fn may_write(&self, uid: &UserId, gids: &[GroupId]) -> bool {
        uid.is_privileged()
            || ((self.uid == *uid) && self.mode.user_write())
            || (gids.iter().any(|gid| self.gid == *gid) && self.mode.group_write())
            || self.mode.other_write()
    }

    /// Return `true` if the supplied UID and GIDs have execute permission on this node. A
    /// privileged user (UID 0) has access if any executable bit is set.
    pub fn may_execute(&self, uid: &UserId, gids: &[GroupId]) -> bool {
        uid.is_privileged() && self.mode.any_execute()
            || ((self.uid == *uid) && self.mode.user_execute())
            || (gids.iter().any(|gid| self.gid == *gid) && self.mode.group_execute())
            || self.mode.other_execute()
    }

    /// If the supplied UID is the owner of the Node, change the Node owner. A privileged user may
    /// always change the Node owner.
    pub fn modify_uid(&mut self, uid: &UserId, new_uid: UserId) {
        if uid.is_privileged() || (self.uid == *uid) {
            self.modified = Utc::now();
            self.uid = new_uid;
        }
    }

    /// If the supplied UID is the owner of the Node, change the Node GID. A privileged user may
    /// always change the Node GID.
    pub fn modify_gid(&mut self, uid: &UserId, new_gid: GroupId) {
        if uid.is_privileged() || (self.uid == *uid) {
            self.modified = Utc::now();
            self.gid = new_gid;
        }
    }

    /// If the supplied UID is the owner of the Node, change the Node permissions. A privileged
    /// user may always change the Node permissions.
    pub fn modify_mode(&mut self, uid: &UserId, new_mode: Mode) {
        if uid.is_privileged() || (self.uid == *uid) {
            self.modified = Utc::now();
            self.mode = new_mode;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn new() {
        let _: Node = Node::new(None, 0, 0, 0);
    }

    proptest! {
        #[test]
        fn privileged_user_may_always_read(nuid in 0u32..65535, ngid in 0u32..65535, nmode in 0u16..0o777, gid in 0u32..65535) {
            let n = Node::new(None, nuid, ngid, nmode);
            prop_assert_eq!(n.may_read(&UserId::privileged(), &[GroupId::from(gid)]), true);
        }

        #[test]
        fn privileged_user_may_always_write(nuid in 0u32..65535, ngid in 0u32..65535, nmode in 0u16..0o777, gid in 0u32..65535) {
            let n = Node::new(None, nuid, ngid, nmode);
            prop_assert_eq!(n.may_write(&UserId::privileged(), &[GroupId::from(gid)]), true);
        }

        #[test]
        fn privileged_user_may_execute_any_executable(nuid in 0u32..65535, ngid in 0u32..65535, nmode in 0u16..0o777, gid in 0u32..65535) {
            let n = Node::new(None, nuid, ngid, nmode);
            prop_assert_eq!(n.may_execute(&UserId::privileged(), &[GroupId::from(gid)]), (nmode & 0o111) > 0);
        }

        #[test]
        fn user_may_read_own_readable(ngid in 1u32..65535, uid in 1u32..65535, gid in 1u32..65535) {
            prop_assert!(Node::new(None, uid, ngid, 0o400).may_read(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o500).may_read(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o600).may_read(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o700).may_read(&UserId::from(uid), &[GroupId::from(gid)]));
        }

        #[test]
        fn user_may_write_own_writable(ngid in 1u32..65535, uid in 1u32..65535, gid in 1u32..65535) {
            prop_assert!(Node::new(None, uid, ngid, 0o200).may_write(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o300).may_write(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o600).may_write(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o700).may_write(&UserId::from(uid), &[GroupId::from(gid)]));
        }

        #[test]
        fn user_may_execute_own_executable(ngid in 1u32..65535, uid in 1u32..65535, gid in 1u32..65535) {
            prop_assert!(Node::new(None, uid, ngid, 0o100).may_execute(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o300).may_execute(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o500).may_execute(&UserId::from(uid), &[GroupId::from(gid)]));
            prop_assert!(Node::new(None, uid, ngid, 0o700).may_execute(&UserId::from(uid), &[GroupId::from(gid)]));
        }
    }
}
