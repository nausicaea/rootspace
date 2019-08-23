use std::time::Instant;
use super::{user_id::UserId, group_id::GroupId, mode::Mode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    name: String,
    uid: UserId,
    gid: GroupId,
    mode: Mode,
    accessed: Instant,
    modified: Instant,
    changed: Instant,
}

impl Node {
    pub fn new<S: AsRef<str>, U: Into<UserId>, G: Into<GroupId>, M: Into<Mode>>(name: S, uid: U, gid: G, mode: M) -> Self {
        let zero = Instant::now();

        Node {
            name: name.as_ref().to_owned(),
            uid: uid.into(),
            gid: gid.into(),
            mode: mode.into(),
            accessed: zero,
            modified: zero,
            changed: zero,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return `true` if the supplied UID and GIDs have read permission on this node. A privileged
    /// user (UID 0) always has read access.
    pub fn may_read(&self, uid: &UserId, gids: &[GroupId]) -> bool {
        let privileged = self.uid.privileged();
        let user_perm = self.mode.user_read() && (self.uid == *uid);
        let group_perm = self.mode.group_read() && gids.iter().any(|gid| self.gid == *gid);
        let other_perm = self.mode.other_read();

        privileged || user_perm || group_perm || other_perm
    }

    /// Return `true` if the supplied UID and GIDs have write permission on this node. A privileged
    /// user (UID 0) always has write access.
    pub fn may_write(&self, uid: &UserId, gids: &[GroupId]) -> bool {
        let privileged = self.uid.privileged();
        let user_perm = self.mode.user_write() && (self.uid == *uid);
        let group_perm = self.mode.group_write() && gids.iter().any(|gid| self.gid == *gid);
        let other_perm = self.mode.other_write();

        privileged || user_perm || group_perm || other_perm
    }

    /// Return `true` if the supplied UID and GIDs have execute permission on this node. A
    /// privileged user (UID 0) has access if any executable bit is set.
    pub fn may_execute(&self, uid: &UserId, gids: &[GroupId]) -> bool {
        let privileged = self.mode.any_execute() && self.uid.privileged();
        let user_perm = self.mode.user_execute() && (self.uid == *uid);
        let group_perm = self.mode.group_execute() && gids.iter().any(|gid| self.gid == *gid);
        let other_perm = self.mode.other_execute();

        privileged || user_perm || group_perm || other_perm
    }

    /// If the supplied UID is the owner of the Node, change the Node owner. A privileged user may
    /// always change the Node owner.
    pub fn modify_uid(&mut self, uid: &UserId, new_uid: UserId) {
        if uid.privileged() || (self.uid == *uid) {
            self.modified = Instant::now();
            self.uid = new_uid;
        }
    }

    /// If the supplied UID is the owner of the Node, change the Node GID. A privileged user may
    /// always change the Node GID.
    pub fn modify_gid(&mut self, uid: &UserId, new_gid: GroupId) {
        if uid.privileged() || (self.uid == *uid) {
            self.modified = Instant::now();
            self.gid = new_gid;
        }
    }

    /// If the supplied UID is the owner of the Node, change the Node permissions. A privileged
    /// user may always change the Node permissions.
    pub fn modify_mode(&mut self, uid: &UserId, new_mode: Mode) {
        if uid.privileged() || (self.uid == *uid) {
            self.modified = Instant::now();
            self.mode = new_mode;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _: Node = Node::new("", 0, 0, 0);
    }

    #[quickcheck]
    fn may_read(nuid: u32, ngid: u32, nmode: u32, uid: u32, gid: u32) -> bool {
        let nu = UserId::from(nuid);
        let ng = GroupId::from(ngid);
        let nm = Mode::from(nmode);
        let u = UserId::from(uid);
        let g = GroupId::from(gid);

        let privileged = u.privileged();
        let user_perm = nm.user_read() && (nu == u);
        let group_perm = nm.group_read() && (ng == g);
        let other_perm = nm.other_read();
        let expected = privileged || user_perm || group_perm || other_perm;

        let n = Node::new("", nuid, ngid, nmode);
        n.may_read(&u, &[g]) == expected
    }

    #[quickcheck]
    fn may_write(nuid: u32, ngid: u32, nmode: u32, uid: u32, gid: u32) -> bool {
        let nu = UserId::from(nuid);
        let ng = GroupId::from(ngid);
        let nm = Mode::from(nmode);
        let u = UserId::from(uid);
        let g = GroupId::from(gid);

        let privileged = u.privileged();
        let user_perm = nm.user_write() && (nu == u);
        let group_perm = nm.group_write() && (ng == g);
        let other_perm = nm.other_write();
        let expected = privileged || user_perm || group_perm || other_perm;

        let n = Node::new("", nuid, ngid, nmode);
        n.may_write(&u, &[g]) == expected
    }

    #[quickcheck]
    fn may_execute(nuid: u32, ngid: u32, nmode: u32, uid: u32, gid: u32) -> bool {
        let nu = UserId::from(nuid);
        let ng = GroupId::from(ngid);
        let nm = Mode::from(nmode);
        let u = UserId::from(uid);
        let g = GroupId::from(gid);

        let privileged = nm.any_execute() && u.privileged();
        let user_perm = nm.user_execute() && (nu == u);
        let group_perm = nm.group_execute() && (ng == g);
        let other_perm = nm.other_execute();
        let expected = privileged || user_perm || group_perm || other_perm;

        let n = Node::new("", nuid, ngid, nmode);
        n.may_execute(&u, &[g]) == expected
    }
}
