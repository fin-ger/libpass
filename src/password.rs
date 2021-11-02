use std::fs;
use std::path::Path;
use std::path::PathBuf;

use id_tree::RemoveBehavior;
use id_tree::NodeId;

use crate::IntoStoreError;
use crate::{DecryptedPassword, Directory, MutEntry, PassNode, Store, StoreError};

#[derive(Debug)]
pub struct Password {
    name: String,
    path: PathBuf,
    node_id: NodeId,
    root: PathBuf
}

impl Password {
    pub(crate) fn new(name: String, path: PathBuf, root: PathBuf, node_id: NodeId) -> Self {
        Self {
            name,
            path,
            root,
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
            self.root.clone(),
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
    store: &'a mut Store,
    node_id: NodeId,
}

impl<'a> MutPassword<'a> {
    pub(crate) fn new(
        node_id: NodeId,
        store: &'a mut Store,
    ) -> Self {
        Self {
            node_id,
            store,
        }
    }

    fn to_entry(&'_ mut self) -> MutEntry<'_> {
        MutEntry::new(self.node_id.clone(), self.store)
    }

    fn data(&self) -> &PassNode {
        self.store.tree.get(&self.node_id).unwrap().data()
    }

    pub fn name(&self) -> &str {
        &self.data().name()
    }

    pub fn path(&self) -> &Path {
        &self.data().path()
    }

    pub fn parent(&self) -> Directory {
        let parent_id = self
            .store
            .tree
            .ancestor_ids(&self.node_id)
            .expect("Password node does not exist in internal tree")
            .next()
            .expect("Password has no parents");
        let parent = self
            .store
            .tree
            .get(&parent_id)
            .expect("Parent of password does not exist in internal tree");

        Directory::new(
            parent.data().name().to_owned(),
            parent.data().path().to_owned(),
            self.store.location().to_owned(),
            parent_id.clone(),
        )
    }

    pub fn remove(self) -> Result<(), StoreError> {
        let path = self.path().to_owned();

        fs::remove_file(&path)
            .with_store_error("Could not remove password")?;
        self.store.tree.remove_node(self.node_id, RemoveBehavior::DropChildren)
            .expect("Could not remove password from internal tree structure");

        let root = self.store.location().to_owned();
        if let Some(git) = self.store.git() {
            git.add(&[&path]).with_store_error("failed to add removal to git")?;
            git.commit(&format!(
                "Remove '{}' from store.",
                path.strip_prefix(root).unwrap().with_extension("").display(),
            )).with_store_error("failed to commit removal to git")?;
        }

        Ok(())
    }

    pub fn rename<N: Into<String>>(&mut self, name: N) -> Result<(), StoreError> {
        self.to_entry().rename(name)
    }

    pub fn move_to(&mut self, directory: &Directory) {
        self.to_entry().move_to(directory)
    }

    pub fn copy_to(&mut self, directory: &Directory) {
        self.to_entry().copy_to(directory)
    }

    pub fn decrypt(&self) -> Result<DecryptedPassword, StoreError> {
        DecryptedPassword::from_path(&self.data().path())
    }

    pub fn make_immut(self) -> Password {
        Password::new(
            self.name().to_owned(),
            self.path().to_owned(),
            self.store.location().to_owned(),
            self.node_id,
        )
    }
}
