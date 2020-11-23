use id_tree::{Tree, NodeId, ChildrenIds};

use std::path::Path;
use std::fmt;

use crate::{Password, Directory, PassNode};

pub struct DirectoryEntry<'a> {
    data: &'a PassNode,
    tree: &'a Tree<PassNode>,
    node_id: NodeId,
}

impl<'a> DirectoryEntry<'a> {
    pub(crate) fn new(node_id: NodeId, tree: &'a Tree<PassNode>) -> Self {
        Self {
            data: tree.get(&node_id).unwrap().data(),
            tree,
            node_id,
        }
    }

    pub fn name(&self) -> &'a str {
        self.data.name()
    }

    pub fn path(&self) -> &'a Path {
        self.data.path()
    }

    pub fn is_dir(&self) -> bool {
        self.data.is_dir()
    }

    pub fn is_password(&self) -> bool {
        self.data.is_password()
    }

    pub fn password(self) -> Option<Password<'a>> {
        if let PassNode::Password { name, path } = self.data {
            Some(Password::new(name, path))
        } else {
            None
        }
    }

    pub fn directory(self) -> Option<Directory<'a>> {
        if let PassNode::Directory { name, path } = self.data {
            Some(Directory::new(name, path, self.tree, &self.node_id))
        } else {
            None
        }
    }
}

impl<'a> fmt::Debug for DirectoryEntry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.is_dir() { "Directory" } else { "Password" };
        f.debug_struct("DirectoryEntry")
            .field("kind", &kind.to_string())
            .field("name", &self.name().to_string())
            .field("path", &self.path().display().to_string())
            .finish()
    }
}

pub struct Entries<'a> {
    tree: &'a Tree<PassNode>,
    iter: ChildrenIds<'a>,
}

impl<'a> Entries<'a> {
    pub(crate) fn new(
        tree: &'a Tree<PassNode>,
        iter: ChildrenIds<'a>,
    ) -> Self {
        Self {
            tree,
            iter,
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = DirectoryEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
            .map(|id| {
                DirectoryEntry::new(id.clone(), self.tree)
            })
    }
}
