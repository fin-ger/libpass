use std::path::Path;
use std::path::PathBuf;

use id_tree::{NodeId, Tree};

use crate::{DecryptedPassword, PassNode, Store, StoreError};

pub struct Password {
    name: String,
    path: PathBuf,
    node_id: NodeId,
}

impl Password {
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

    pub fn decrypt(&self) -> Result<DecryptedPassword, StoreError> {
        DecryptedPassword::from_path(&self.path)
    }

    pub fn make_mut(self, store: &mut Store) -> MutPassword {
        store.mut_password(self)
    }
}

pub struct MutPassword<'a> {
    name: String,
    path: PathBuf,
    tree: &'a mut Tree<PassNode>,
    node_id: NodeId,
}

impl<'a> MutPassword<'a> {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn decrypt(&self) -> Result<DecryptedPassword, StoreError> {
        DecryptedPassword::from_path(&self.path)
    }
}
