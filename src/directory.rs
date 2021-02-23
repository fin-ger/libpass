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

    pub fn password_insertion<N: Into<String>>(&mut self, name: N) -> PasswordInserter {
        let name = name.into();
        let path = self.path.join(&name);
        PasswordInserter::new(self.node_id.clone(), path, name)
    }

    #[cfg(feature = "parsed-passwords")]
    pub fn parsed_password_insertion<N: Into<String>>(
        &mut self,
        name: N,
    ) -> crate::parsed::PasswordInserter {
        let name = name.into();
        let path = self.path.join(&name);
        crate::parsed::PasswordInserter::new(self.node_id.clone(), path, name)
    }

    pub fn directory_insertion<N: Into<String>>(&mut self, name: N) -> DirectoryInserter {
        let name = name.into();
        let path = self.path.join(&name);
        DirectoryInserter::new(path, name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /*pub fn gpg_ids(&self) -> impl Iterator<Item = &str> {
        todo!();
    }*/

    pub fn parent(&self, store: &Store) -> Option<Directory> {
        let parent_id = store.tree().ancestor_ids(&self.node_id).ok()?.next()?;
        let parent = store.tree().get(&parent_id).ok()?;

        Some(Directory::new(
            parent.data().name().to_owned(),
            parent.data().path().to_owned(),
            parent_id.clone(),
        ))
    }

    pub(crate) fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn make_mut(self, store: &mut Store) -> MutDirectory {
        store.mut_directory(self)
    }
}

pub enum OpMode {
    Default,
    Recursive,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /*pub fn gpg_ids(&self) -> impl Iterator<Item = &str> {
        todo!();
    }*/

    pub fn add_gpg_id(&self, gpg_id: &str) {
        todo!();
    }

    pub fn remove_gpg_id(&self, gpg_id: &str) {
        todo!();
    }

    pub fn clear_gpg_ids(&self, gpg_id: &str) {
        todo!();
    }

    pub fn set_gpg_ids(&self, gpg_ids: Vec<&str>) {
        todo!();
    }

    pub fn parent(&self) -> Option<Directory> {
        let parent_id = self.tree.ancestor_ids(&self.node_id).ok()?.next()?;
        let parent = self.tree.get(&parent_id).ok()?;

        Some(Directory::new(
            parent.data().name().to_owned(),
            parent.data().path().to_owned(),
            parent_id.clone(),
        ))
    }

    pub fn remove(self, _op_mode: OpMode) {
        todo!();
    }

    pub fn rename<N: Into<String>>(&mut self, _name: N) {
        todo!();
    }

    pub fn move_to(&mut self, _directory: &Directory) {
        todo!();
    }

    pub fn copy_to(&mut self, _directory: &Directory) {
        todo!();
    }
}
