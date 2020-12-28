mod group_id;
mod mode;
mod node;
mod user_id;

use serde_json;
use anyhow::Result;
use file_manipulation::{FilePathBuf, NewOrExFilePathBuf};
use self::{group_id::GroupId, mode::Mode, node::Node, user_id::UserId};
use engine::{AssetTrait, AssetMutTrait};
use bitflags::bitflags;
use daggy::{Dag, NodeIndex, Walker};
use std::collections::HashMap;
use thiserror::Error;
use std::ffi::OsStr;
use std::path::{Path, Component};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::convert::TryFrom;

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

bitflags!{
    pub struct AccessMode: u8 {
        const READ_OK = 0x2;
        const WRITE_OK = 0x4;
        const EXECUTE_OK = 0x8;
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessData {
    uid: UserId,
    gids: Vec<GroupId>,
}

impl ProcessData {
    fn new(uid: UserId, gids: &[GroupId]) -> Self {
        ProcessData {
            uid,
            gids: gids.iter().copied().collect(),
        }
    }
}

#[derive(Deserialize, Serialize)]
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

    pub fn access(&self, path: &Path, how: AccessMode, process_data: &ProcessData) -> Result<(), Error> {
        self.for_node(path, process_data, |_, node| {
            let fail_access = (how.intersects(AccessMode::READ_OK) && !node.may_read(process_data))
                || (how.intersects(AccessMode::WRITE_OK) && !node.may_write(process_data))
                || (how.intersects(AccessMode::EXECUTE_OK) && !node.may_execute(process_data));

            if fail_access {
                Err(Error::NoPermission)
            } else {
                Ok(())
            }
        })
    }

    pub fn stat(&self, path: &Path, process_data: &ProcessData) -> Result<&Node, Error> {
        self.for_node(path, process_data, |_, node| Ok(node))
    }

    pub fn read(&self, path: &Path, process_data: &ProcessData) -> Result<&[u8], Error> {
        self.for_node(path, process_data, |idx, _| Ok(self.data[&idx].as_slice()))
    }

    fn for_node<'a, T, F>(&'a self, path: &Path, process_data: &ProcessData, op: F) -> Result<T, Error>
    where
        F: Fn(NodeIndex, &'a Node) -> Result<T, Error>,
    {
        let components: Vec<_> = path.components().collect();
        let num_segments = components.len();

        let mut parent_node = self.root_idx;

        for (i, component) in components.iter().enumerate() {
            if let Component::Normal(node_name) = component {
                let child_node = self.find_child(parent_node, node_name, process_data)?;

                if i == num_segments - 1 {
                    let node = self.graph.node_weight(child_node).ok_or(Error::NodeNotFound)?;

                    return op(child_node, node);
                }

                parent_node = child_node;
            } else {
                return Err(Error::NotADirectory);
            }
        }

        Err(Error::NodeNotFound)
    }

    fn find_child(&self, parent: NodeIndex, child_name: &OsStr, process_data: &ProcessData) -> Result<NodeIndex, Error> {
        let parent_node = self.graph.node_weight(parent).ok_or(Error::NodeNotFound)?;

        if !parent_node.may_execute(process_data) {
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

impl AssetTrait for FileSystem {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let fp = FilePathBuf::try_from(path.as_ref())?;
        let mut file = File::open(fp)?;
        let mut de = serde_json::Deserializer::from_reader(&mut file);
        let fs = FileSystem::deserialize(&mut de)?;
        Ok(fs)
    }
}

impl AssetMutTrait for FileSystem {
    fn to_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let nfp = NewOrExFilePathBuf::try_from(path.as_ref())?;
        let mut file = File::create(nfp)?;
        let mut ser = serde_json::Serializer::pretty(&mut file);
        self.serialize(&mut ser)?;
        Ok(())
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
        let _: FileSystem = FileSystem::builder()
            .with_mode_mask(Mode::from(0o012))
            .build();
    }

    #[test]
    fn num_nodes_empty_fs() {
        let fs = FileSystem::default();
        assert_eq!(fs.num_nodes(), 1);
    }
}
