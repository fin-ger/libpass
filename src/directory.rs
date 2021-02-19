use id_tree::{NodeId, Tree};

use std::path::{Path, PathBuf};

use crate::{DirectoryInserter, PassNode, PasswordInserter, Store};

pub struct Directory {
    name: String,
    path: PathBuf,
    node_id: NodeId,
}

impl Directory {
    pub(crate) fn new(name: String, path: PathBuf, node_id: NodeId) -> Self {
        Self {
            name,
            path,
            node_id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn node_id(&self) -> &NodeId {
        &self.node_id
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
        node_id: NodeId,
    ) -> Self {
        Self {
            name,
            path,
            tree,
            node_id,
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
}
