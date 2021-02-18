use id_tree::{NodeId, Tree};

use std::path::{Path, PathBuf};

use crate::{
    Directories, DirectoryInserter, Entries, PassNode, PasswordInserter, Passwords, Store,
};

pub struct Directory<'a> {
    name: &'a str,
    path: &'a Path,
    tree: &'a Tree<PassNode>,
    node_id: NodeId,
}

impl<'a> Directory<'a> {
    pub(crate) fn new(
        name: &'a str,
        path: &'a Path,
        tree: &'a Tree<PassNode>,
        node: NodeId,
    ) -> Self {
        Self {
            name,
            path,
            tree,
            node_id: node,
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn path(&self) -> &'a Path {
        self.path
    }

    pub(crate) fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn passwords(&self) -> Passwords {
        Passwords::new(self.entries())
    }

    pub fn directories(&self) -> Directories {
        Directories::new(self.entries())
    }

    pub fn entries(&self) -> Entries {
        let entries = self
            .tree
            .children_ids(&self.node_id)
            .expect("Failed to read directory entries from internal tree");
        Entries::new(self.tree, entries)
    }

    pub fn make_mut(self, store: &mut Store) -> MutDirectory {
        store.mut_directory(self)
    }
}

pub struct MutDirectory<'a> {
    name: String,
    path: PathBuf,
    tree: &'a mut Tree<PassNode>,
    node_id: NodeId,
}

impl<'a> MutDirectory<'a> {
    pub(crate) fn new(
        name: String,
        path: PathBuf,
        tree: &'a mut Tree<PassNode>,
        node: NodeId,
    ) -> Self {
        Self {
            name,
            path,
            tree,
            node_id: node,
        }
    }

    pub fn password_insertion<N: Into<String>>(&mut self, name: N) -> PasswordInserter {
        let name = name.into();
        let path = self.path.join(&name);
        PasswordInserter::new(self.tree, path, name)
    }

    pub fn directory_insertion<N: Into<String>>(&mut self, name: N) -> DirectoryInserter {
        let name = name.into();
        let path = self.path.join(&name);
        DirectoryInserter::new(self.tree, path, name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn passwords(&self) -> Passwords {
        Passwords::new(self.entries())
    }

    pub fn directories(&self) -> Directories {
        Directories::new(self.entries())
    }

    pub fn entries(&self) -> Entries {
        let entries = self
            .tree
            .children_ids(&self.node_id)
            .expect("Failed to read directory entries from internal tree");
        Entries::new(self.tree, entries)
    }
}
