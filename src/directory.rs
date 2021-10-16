use gpgme::{Context, Protocol};
use id_tree::{NodeId, Tree};

use std::{
    fs::OpenOptions,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{DirectoryInserter, IntoStoreError, PassNode, PasswordInserter, Store, StoreError};

pub struct Directory {
    name: String,
    path: PathBuf,
    node_id: NodeId,
    root: PathBuf,
}

impl Directory {
    pub(crate) fn new(name: String, path: PathBuf, root: PathBuf, node_id: NodeId) -> Self {
        Self {
            name,
            path,
            node_id,
            root,
        }
    }

    pub fn password_insertion<N: Into<String>>(&mut self, name: N) -> PasswordInserter {
        let name = name.into();
        let path = self.path.join(format!("{}.gpg", name));
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
        DirectoryInserter::new(self.node_id.clone(), path, name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn gpg_ids(&self) -> Result<Vec<String>, StoreError> {
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)
            .with_store_error("creating OpenPGP context")?;
        let mut path = self.path.as_path();
        loop {
            let gpg_id = path.join(".gpg-id");
            if gpg_id.is_file() {
                let mut file = OpenOptions::new()
                    .read(true)
                    .open(&gpg_id)
                    .with_store_error(gpg_id.display().to_string())?;
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .with_store_error(gpg_id.display().to_string())?;

                return content
                    .lines()
                    .map(|line| {
                        Ok(ctx
                            .get_key(line)
                            .with_store_error("GPG ID not found")?
                            .id()
                            .expect("GPG ID not valid utf-8")
                            .to_string())
                    })
                    .collect::<Result<Vec<String>, StoreError>>();
            }

            if let Some(parent) = path.parent() {
                if path.starts_with(&self.root) {
                    path = parent
                } else {
                    return Err(StoreError::NoGpgId(self.path.display().to_string()));
                }
            } else {
                return Err(StoreError::NoGpgId(self.path.display().to_string()))
            }
        }
    }

    pub fn parent(&self, store: &Store) -> Option<Directory> {
        let parent_id = store.tree().ancestor_ids(&self.node_id).ok()?.next()?;
        let parent = store.tree().get(&parent_id).ok()?;

        Some(Directory::new(
            parent.data().name().to_owned(),
            parent.data().path().to_owned(),
            self.root.clone(),
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
    root: PathBuf,
    node_id: NodeId,
}

impl<'a> MutDirectory<'a> {
    pub(crate) fn new(
        name: String,
        path: PathBuf,
        tree: &'a mut Tree<PassNode>,
        root: PathBuf,
        node_id: NodeId,
    ) -> Self {
        Self {
            name,
            path,
            tree,
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
            self.root.clone(),
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
