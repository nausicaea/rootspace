mod user_id;
mod group_id;
mod mode;
mod node;

use self::node::Node;
use self::mode::Mode;
use self::user_id::UserId;
use self::group_id::GroupId;
use std::fmt;
use daggy::{Dag, NodeIndex, Walker};

#[derive(Debug)]
pub enum Error {
    NodeNotFound,
    NoPermission,
    FileNotFound,
    NotADirectory,
}

pub struct FileSystem {
    graph: Dag<Node, Option<Vec<u8>>>,
    root_idx: NodeIndex,
    separator: char,
    mode_mask: Mode,
}

impl FileSystem {
    pub fn nodes(&self) -> usize {
        self.graph.node_count()
    }

    fn child<S>(&self, parent: NodeIndex, child_name: S, uid: &UserId, gids: &[GroupId]) -> Result<NodeIndex, Error>
    where
        S: AsRef<str>,
    {
        let parent_node = self.graph.node_weight(parent)
            .ok_or(Error::NodeNotFound)?;

        if parent_node.may_execute(uid, gids) {
            let mut children = self.graph.children(parent);
            while let Some((_, child)) = children.walk_next(&self.graph) {
                let child_node = self.graph.node_weight(child)
                    .ok_or(Error::NodeNotFound)?;

                if child_node.name() == child_name.as_ref() {
                    return Ok(child);
                }
            }

            Err(Error::NodeNotFound)
        } else {
            Err(Error::NoPermission)
        }
    }

    fn find<S>(&self, path: S, uid: &UserId, gids: &[GroupId]) -> Result<NodeIndex, Error>
    where
        S: AsRef<str>,
    {
        let segments: Vec<&str> = path.as_ref().split(self.separator).collect();
        let num_segments = segments.len();
        let mut parent_node = self.root_idx;
        for (i, node_name) in segments.iter().enumerate() {
            let child_node = self.child(parent_node, node_name, uid, gids)?;
            if i == num_segments - 1 {
                return Ok(child_node);
            } else {
                parent_node = child_node;
            }
        }

        Err(Error::NodeNotFound)
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        let mut graph = Dag::default();
        let root_idx = graph.add_node(Node::new("/", 0, 0, 0o755));

        FileSystem {
            graph,
            root_idx,
            separator: '/',
            mode_mask: Mode::from(0o022),
        }
    }
}

impl fmt::Debug for FileSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FileSystem(#nodes: {})", self.graph.node_count())
    }
}
