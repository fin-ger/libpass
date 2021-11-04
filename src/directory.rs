use gpgme::{Context, Protocol};
use id_tree::{NodeId, RemoveBehavior};

use std::{fs::{self, OpenOptions}, io::Read, path::{Path, PathBuf}};

use crate::{DirectoryInserter, IntoStoreError, MutEntry, Traversal, PassNode, PasswordInserter, Store, StoreError};

#[derive(Debug)]
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

    pub fn password_insertion<N: Into<String>>(&self, name: N) -> PasswordInserter {
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

    pub fn directory_insertion<N: Into<String>>(&self, name: N) -> DirectoryInserter {
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

    pub fn parent(self, store: &Store) -> Option<Directory> {
        // consume self here, to avoid a parent directory being removed and
        // having references to node_ids of child directory entries
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

pub struct MutDirectory<'a> {
    store: &'a mut Store,
    node_id: NodeId,
}

impl<'a> MutDirectory<'a> {
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

    /*pub fn gpg_ids(&self) -> impl Iterator<Item = &str> {
        todo!();
    }*/

    pub fn add_gpg_id(&self, _gpg_id: &str) {
        todo!();
    }

    pub fn remove_gpg_id(&self, _gpg_id: &str) {
        todo!();
    }

    pub fn clear_gpg_ids(&self, _gpg_id: &str) {
        todo!();
    }

    pub fn set_gpg_ids(&self, _gpg_ids: Vec<&str>) {
        todo!();
    }

    pub fn parent(self) -> Option<Directory> {
        // consume self here, to avoid a parent directory being removed and
        // having references to node_ids of child directory entries
        let parent_id = self.store.tree.ancestor_ids(&self.node_id).ok()?.next()?;
        let parent = self.store.tree.get(&parent_id).ok()?;

        Some(Directory::new(
            parent.data().name().to_owned(),
            parent.data().path().to_owned(),
            self.store.location().to_owned(),
            parent_id.clone(),
        ))
    }

    pub fn remove(self, traversal: Traversal) -> Result<(), StoreError> {
        let path = self.path().to_owned();

        match traversal {
            Traversal::None => {
                fs::remove_dir(&path)
                    .with_store_error("Could not remove directory as it is not empty")?;
            }
            Traversal::Recursive => {
                fs::remove_dir_all(&path)
                    .with_store_error("Could not remove directory recursively")?;
            }
        }
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

    pub fn make_immut(self) -> Directory {
        Directory::new(self.name().into(), self.path().into(), self.store.location().into(), self.node_id)
    }
}
