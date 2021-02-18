use id_tree::{NodeId, Tree};

use std::fmt;
use std::path::Path;

use crate::{Directory, MutDirectory, MutPassword, PassNode, Password, Store};

pub struct Entry<'a> {
    data: &'a PassNode,
    tree: &'a Tree<PassNode>,
    node_id: NodeId,
}

impl<'a> Entry<'a> {
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

    pub(crate) fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn is_dir(&self) -> bool {
        self.data.is_dir()
    }

    pub fn is_password(&self) -> bool {
        self.data.is_password()
    }

    pub fn password(&self) -> Option<Password<'a>> {
        if let PassNode::Password { name, path } = self.data {
            Some(Password::new(name, path, self.node_id.clone()))
        } else {
            None
        }
    }

    pub fn directory(&self) -> Option<Directory<'a>> {
        if let PassNode::Directory { name, path } = self.data {
            Some(Directory::new(name, path, self.tree, self.node_id.clone()))
        } else {
            None
        }
    }

    pub fn make_mut(self, store: &mut Store) -> MutEntry {
        store.mut_entry(self)
    }
}

impl<'a> fmt::Debug for Entry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.is_dir() {
            "Directory"
        } else {
            "Password"
        };
        f.debug_struct("Entry")
            .field("kind", &kind.to_string())
            .field("name", &self.name().to_string())
            .field("path", &self.path().display().to_string())
            .finish()
    }
}

pub struct MutEntry<'a> {
    data: PassNode,
    tree: &'a mut Tree<PassNode>,
    node_id: NodeId,
}

impl<'a> MutEntry<'a> {
    pub(crate) fn new(node_id: NodeId, tree: &'a mut Tree<PassNode>) -> Self {
        Self {
            data: tree.get(&node_id).unwrap().data().clone(),
            tree,
            node_id,
        }
    }

    pub fn name(&self) -> &str {
        self.data.name()
    }

    pub fn path(&self) -> &Path {
        self.data.path()
    }

    pub fn is_dir(&self) -> bool {
        self.data.is_dir()
    }

    pub fn is_password(&self) -> bool {
        self.data.is_password()
    }

    pub fn mut_password(self) -> Option<MutPassword<'a>> {
        if let PassNode::Password { name, path } = self.data {
            Some(MutPassword::new(name, path, self.tree, self.node_id))
        } else {
            None
        }
    }

    pub fn mut_directory(self) -> Option<MutDirectory<'a>> {
        if let PassNode::Directory { name, path } = self.data {
            Some(MutDirectory::new(name, path, self.tree, self.node_id))
        } else {
            None
        }
    }
}

impl<'a> fmt::Debug for MutEntry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.is_dir() {
            "Directory"
        } else {
            "Password"
        };
        f.debug_struct("MutEntry")
            .field("kind", &kind.to_string())
            .field("name", &self.name().to_string())
            .field("path", &self.path().display().to_string())
            .finish()
    }
}
