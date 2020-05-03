mod group_id;
mod mode;
mod node;
mod user_id;

use self::{group_id::GroupId, mode::Mode, node::Node, user_id::UserId};
use daggy::{Dag, NodeIndex, Walker};
use std::collections::HashMap;
use thiserror::Error;
use std::ffi::OsStr;
use std::path::{Path, Component};

#[derive(Debug, Error)]
pub enum Error {
    #[error("No such node was found")]
    NodeNotFound,
    #[error("Insufficient permission to access that node")]
    NoPermission,
    #[error("The file node was not found")]
    FileNotFound,
    #[error("The node is not a directory")]
    NotADirectory,
}

pub struct FileSystem {
    graph: Dag<Node, ()>,
    root_idx: NodeIndex,
    mode_mask: Mode,
    data: HashMap<NodeIndex, Vec<u8>>,
}

impl FileSystem {
    pub fn new() -> FileSystem {
        FileSystem::default()
    }

    pub fn builder() -> FileSystemBuilder {
        FileSystemBuilder::default()
    }

    pub fn num_nodes(&self) -> usize {
        self.graph.node_count()
    }

    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    pub fn stat(&self, path: &Path, uid: &UserId, gids: &[GroupId]) -> Result<&Node, Error> {
        let components: Vec<_> = path.components().collect();
        let num_segments = components.len();
        let mut parent_node = self.root_idx;
        for (i, component) in components.iter().enumerate() {
            if let Component::Normal(node_name) = component {
                let child_node = self.find_child(parent_node, node_name, uid, gids)?;

                if i == num_segments - 1 {
                    return self.graph.node_weight(child_node).ok_or(Error::NodeNotFound);
                }

                parent_node = child_node;
            }
        }

        Err(Error::NodeNotFound)
    }

    fn find_child(&self, parent: NodeIndex, child_name: &OsStr, uid: &UserId, gids: &[GroupId]) -> Result<NodeIndex, Error> {
        let parent_node = self.graph.node_weight(parent).ok_or(Error::NodeNotFound)?;

        if !parent_node.may_execute(uid, gids) {
            return Err(Error::NoPermission);
        }

        let mut children = self.graph.children(parent);
        while let Some((_, child)) = children.walk_next(&self.graph) {
            let child_node = self.graph.node_weight(child).ok_or(Error::NodeNotFound)?;

            if child_node.name() == Some(child_name) {
                return Ok(child);
            }
        }

        Err(Error::NodeNotFound)
    }
}

impl From<FileSystemBuilder> for FileSystem {
    fn from(value: FileSystemBuilder) -> Self {
        let mut graph = Dag::default();
        let root_node = Node::new(
            None,
            value.root_uid,
            value.root_gid,
            Mode::directory(value.mode_mask),
        );
        let root_idx = graph.add_node(root_node);

        FileSystem {
            graph,
            root_idx,
            mode_mask: value.mode_mask,
            data: HashMap::default(),
        }
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl std::fmt::Debug for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FileSystem(mode_mask: {:?}, #nodes: {}, #data: {})", self.mode_mask, self.num_nodes(), self.data_len())
    }
}

#[derive(Debug, Clone)]
pub struct FileSystemBuilder {
    root_uid: UserId,
    root_gid: GroupId,
    mode_mask: Mode,
}

impl FileSystemBuilder {
    pub fn new() -> Self {
        FileSystemBuilder::default()
    }

    pub fn with_mode_mask(mut self, mask: Mode) -> Self {
        self.mode_mask = mask;
        self
    }

    pub fn build(self) -> FileSystem {
        self.into()
    }
}

impl Default for FileSystemBuilder {
    fn default() -> Self {
        let mask = Mode::from(0o022);

        FileSystemBuilder {
            root_uid: UserId::privileged(),
            root_gid: GroupId::privileged(),
            mode_mask: mask,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _: FileSystem = FileSystem::new();
    }

    #[test]
    fn default() {
        let _: FileSystem = Default::default();
    }

    #[test]
    fn builder() {
        let _: FileSystemBuilder = FileSystem::builder();
    }

    #[test]
    fn num_nodes() {
        let fs = FileSystem::default();
        assert_eq!(fs.num_nodes(), 1);
    }
}
