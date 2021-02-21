use std::path::Path;
use std::path::PathBuf;

use id_tree::{NodeId, Tree};

use crate::{DecryptedPassword, Directory, PassNode, Store, StoreError};

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

    pub fn parent(&self, store: &Store) -> Directory {
        let parent_id = store
            .tree()
            .ancestor_ids(&self.node_id)
            .expect("Password node does not exist in internal tree")
            .next()
            .expect("Password has no parents");
        let parent = store
            .tree()
            .get(&parent_id)
            .expect("Parent of password does not exist in internal tree");

        Directory::new(
            parent.data().name().to_owned(),
            parent.data().path().to_owned(),
            parent_id.clone(),
        )
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

    pub fn parent(&self) -> Directory {
        let parent_id = self
            .tree
            .ancestor_ids(&self.node_id)
            .expect("Password node does not exist in internal tree")
            .next()
            .expect("Password has no parents");
        let parent = self
            .tree
            .get(&parent_id)
            .expect("Parent of password does not exist in internal tree");

        Directory::new(
            parent.data().name().to_owned(),
            parent.data().path().to_owned(),
            parent_id.clone(),
        )
    }

    pub fn remove(self) {}

    pub fn rename<N: Into<String>>(&mut self, _name: N) {}

    pub fn move_to(&mut self, _directory: &Directory) {}

    pub fn copy_to(&mut self, _directory: &Directory) {}

    pub fn decrypt(&self) -> Result<DecryptedPassword, StoreError> {
        DecryptedPassword::from_path(&self.path)
    }
}
