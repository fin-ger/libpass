use id_tree::{Tree, NodeId, ChildrenIds};

use std::path::Path;

use crate::{PassNode, Passwords, Directories, Entries};

pub struct Directory<'a> {
    name: &'a str,
    path: &'a Path,
    tree: &'a Tree<PassNode>,
    entries: ChildrenIds<'a>,
}

impl<'a> Directory<'a> {
    pub(crate) fn new(
        name: &'a str,
        path: &'a Path,
        tree: &'a Tree<PassNode>,
        node: &NodeId,
    ) -> Self {
        let entries = tree.children_ids(node)
            .expect("Failed to read directory entries from internal tree");

        Self {
            name,
            path,
            tree,
            entries,
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn path(&self) -> &'a Path {
        self.path
    }

    pub fn passwords(&self) -> Passwords {
        Passwords::new(self.entries())
    }

    pub fn directories(&self) -> Directories {
        Directories::new(self.entries())
    }

    pub fn entries(&self) -> Entries {
        Entries::new(self.tree, self.entries.clone())
    }
}
