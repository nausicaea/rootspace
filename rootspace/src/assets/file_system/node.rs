use std::ffi::{OsStr, OsString};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{group_id::GroupId, mode::Mode, user_id::UserId, ProcessData};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
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
        let unix_epoch = DateTime::from(std::time::UNIX_EPOCH);

        Node {
            name: name.map(|n| n.to_owned()),
            uid: uid.into(),
            gid: gid.into(),
            mode: mode.into(),
            accessed: unix_epoch,
            modified: unix_epoch,
            changed: unix_epoch,
        }
    }

    pub fn name(&self) -> Option<&OsStr> {
        self.name.as_deref()
    }

    /// Return `true` if the supplied UID and GIDs have read permission on this node. A privileged
    /// user (UID 0) always has read access.
    pub fn may_read(&self, process_data: &ProcessData) -> bool {
        process_data.uid.is_privileged()
            || ((self.uid == process_data.uid) && self.mode.user_read())
            || (process_data.gids.iter().any(|gid| self.gid == *gid) && self.mode.group_read())
            || self.mode.other_read()
    }

    /// Return `true` if the supplied UID and GIDs have write permission on this node. A privileged
    /// user (UID 0) always has write access.
    pub fn may_write(&self, process_data: &ProcessData) -> bool {
        process_data.uid.is_privileged()
            || ((self.uid == process_data.uid) && self.mode.user_write())
            || (process_data.gids.iter().any(|gid| self.gid == *gid) && self.mode.group_write())
            || self.mode.other_write()
    }

    /// Return `true` if the supplied UID and GIDs have execute permission on this node. A
    /// privileged user (UID 0) has access if any executable bit is set.
    pub fn may_execute(&self, process_data: &ProcessData) -> bool {
        process_data.uid.is_privileged() && self.mode.any_execute()
            || ((self.uid == process_data.uid) && self.mode.user_execute())
            || (process_data.gids.iter().any(|gid| self.gid == *gid) && self.mode.group_execute())
            || self.mode.other_execute()
    }

    /// If the supplied UID is the owner of the Node, change the Node owner. A privileged user may
    /// always change the Node owner.
    pub fn modify_uid(&mut self, process_data: &ProcessData, new_uid: UserId) {
        if process_data.uid.is_privileged() || (self.uid == process_data.uid) {
            self.changed = Utc::now();
            self.uid = new_uid;
        }
    }

    /// If the supplied UID is the owner of the Node, change the Node GID. A privileged user may
    /// always change the Node GID.
    pub fn modify_gid(&mut self, process_data: &ProcessData, new_gid: GroupId) {
        if process_data.uid.is_privileged() || (self.uid == process_data.uid) {
            self.changed = Utc::now();
            self.gid = new_gid;
        }
    }

    /// If the supplied UID is the owner of the Node, change the Node permissions. A privileged
    /// user may always change the Node permissions.
    pub fn modify_mode(&mut self, process_data: &ProcessData, new_mode: Mode) {
        if process_data.uid.is_privileged() || (self.uid == process_data.uid) {
            self.changed = Utc::now();
            self.mode = new_mode;
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn new() {
        let _: Node = Node::new(None, 0, 0, 0);
    }

    proptest! {
        #[test]
        fn privileged_user_may_always_read(nuid in 0u32..65535, ngid in 0u32..65535, nmode in 0u16..0o777, gid in 0u32..65535) {
            let n = Node::new(None, nuid, ngid, nmode);
            let pd = ProcessData::new(UserId::privileged(), &[GroupId::from(gid)]);
            prop_assert_eq!(n.may_read(&pd), true);
        }

        #[test]
        fn privileged_user_may_always_write(nuid in 0u32..65535, ngid in 0u32..65535, nmode in 0u16..0o777, gid in 0u32..65535) {
            let n = Node::new(None, nuid, ngid, nmode);
            let pd = ProcessData::new(UserId::privileged(), &[GroupId::from(gid)]);
            prop_assert_eq!(n.may_write(&pd), true);
        }

        #[test]
        fn privileged_user_may_execute_any_executable(nuid in 0u32..65535, ngid in 0u32..65535, nmode in 0u16..0o777, gid in 0u32..65535) {
            let n = Node::new(None, nuid, ngid, nmode);
            let pd = ProcessData::new(UserId::privileged(), &[GroupId::from(gid)]);
            prop_assert_eq!(n.may_execute(&pd), (nmode & 0o111) > 0);
        }

        #[test]
        fn user_may_read_own_readable(ngid in 1u32..65535, uid in 1u32..65535, gid in 1u32..65535) {
            let pd = ProcessData::new(UserId::from(uid), &[GroupId::from(gid)]);
            prop_assert!(Node::new(None, uid, ngid, 0o400).may_read(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o500).may_read(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o600).may_read(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o700).may_read(&pd));
        }

        #[test]
        fn user_may_write_own_writable(ngid in 1u32..65535, uid in 1u32..65535, gid in 1u32..65535) {
            let pd = ProcessData::new(UserId::from(uid), &[GroupId::from(gid)]);
            prop_assert!(Node::new(None, uid, ngid, 0o200).may_write(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o300).may_write(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o600).may_write(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o700).may_write(&pd));
        }

        #[test]
        fn user_may_execute_own_executable(ngid in 1u32..65535, uid in 1u32..65535, gid in 1u32..65535) {
            let pd = ProcessData::new(UserId::from(uid), &[GroupId::from(gid)]);
            prop_assert!(Node::new(None, uid, ngid, 0o100).may_execute(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o300).may_execute(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o500).may_execute(&pd));
            prop_assert!(Node::new(None, uid, ngid, 0o700).may_execute(&pd));
        }
    }
}
