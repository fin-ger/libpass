use gpgme::{Context, Protocol};
use id_tree::{NodeId, RemoveBehavior};

use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    DirectoryInserter, IntoStoreError, Entry, MutEntry, PassNode, PasswordInserter, Store, StoreError,
    Traversal, TraversalOrder, search_gpg_ids,
};

#[derive(Debug, Clone)]
pub struct GpgKeyId {
    id: String,
    key: gpgme::Key,
}

impl PartialEq for GpgKeyId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for GpgKeyId {}

impl std::hash::Hash for GpgKeyId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl GpgKeyId {
    pub fn new(key_id: impl AsRef<str>) -> gpgme::Result<Self> {
        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let key = ctx.get_key(key_id.as_ref())?;

        Ok(Self {
            id: key_id.as_ref().to_owned(),
            key,
        })
    }

    pub fn key(&self) -> &gpgme::Key {
        &self.key
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct Directory {
    name: String,
    path: PathBuf,
    node_id: NodeId,
    root: PathBuf,
}

fn get_gpg_ids_for_path(path: &Path) -> Result<Vec<GpgKeyId>, StoreError> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)
        .with_store_error("creating OpenPGP context")?;

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
            .map(|id| {
                let key = ctx
                    .get_key(id)
                    .with_store_error("GPG ID not found")?;
                Ok(GpgKeyId { key, id: id.to_string() })
            })
            .collect::<Result<Vec<GpgKeyId>, StoreError>>();
    } else {
        return Err(StoreError::NoGpgId(path.display().to_string()));
    }
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

    pub fn gpg_ids(&self) -> Result<Vec<GpgKeyId>, StoreError> {
        get_gpg_ids_for_path(self.path.as_path())
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
    pub(crate) fn new(node_id: NodeId, store: &'a mut Store) -> Self {
        Self { node_id, store }
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

    fn set_new_gpg_ids_and_reencrypt_passwords(
        &mut self,
        gpg_ids: Vec<GpgKeyId>,
    ) -> Result<(), StoreError> {
        let old_gpg_ids = self.gpg_ids()?;

        let root = self.store.location().to_owned();
        let path = self.path().to_owned();
        let write_gpg_ids = |gpg_ids: &Vec<GpgKeyId>, store: &mut Store| {
            let joined_ids = gpg_ids.iter()
                .map(GpgKeyId::id)
                .collect::<Vec<_>>()
                .join(", ");
            let gpg_id = path.join(".gpg-id").to_path_buf();
            if !gpg_ids.is_empty() {
                let mut file = OpenOptions::new()
                    .write(true)
                    .open(&gpg_id)
                    .with_store_error(gpg_id.display().to_string())?;
                for key in gpg_ids {
                    file.write_all(format!("{}\n", key.id()).as_bytes())
                        .with_store_error(gpg_id.display().to_string())?;
                }
                drop(file);

                if let Some(git) = store.git() {
                    git.add(&[&gpg_id])
                        .with_store_error("failed to add .gpg-id edit to git")?;
                    let name =  path.strip_prefix(&root)
                        .unwrap()
                        .with_extension("")
                        .display()
                        .to_string();
                    if name.is_empty() {
                        git.commit(&format!(
                            "Main GPG IDs for store set to {}.",
                            joined_ids,
                        )).with_store_error("failed to commit .gpg-id change to git")?;
                    } else {
                        git.commit(&format!(
                            "GPG IDs for '{}' set to {}.",
                            name,
                            joined_ids,
                        )).with_store_error("failed to commit .gpg-id change to git")?;
                    }
                }
            } else if gpg_id.exists() {
                let mut ctx = Context::from_protocol(Protocol::OpenPgp)
                    .with_store_error("creating OpenPGP context")?;
                let parent = path.parent().unwrap();
                if parent.starts_with(&root) {
                    let parent_gpg_ids = search_gpg_ids(parent, &mut ctx)?;

                    if parent_gpg_ids.is_empty() {
                        return Err(
                            StoreError::NoGpgId("Cannot clear gpg-ids as this would leave the store without any gpg-ids".to_string())
                        );
                    }

                    fs::remove_file(&gpg_id)
                        .with_store_error("Could not remove gpg-id file")?;

                    if let Some(git) = store.git() {
                        git.add(&[&gpg_id])
                            .with_store_error("failed to add .gpg-id removal to git")?;
                        let name =  path.strip_prefix(&root)
                            .unwrap()
                            .with_extension("")
                            .display()
                            .to_string();
                        if name.is_empty() {
                            git.commit(&format!(
                                "GPG IDs for store removed.",
                            )).with_store_error("failed to commit .gpg-id removal to git")?;
                        } else {
                            git.commit(&format!(
                                "GPG IDs for '{}' removed.",
                                name,
                            )).with_store_error("failed to commit .gpg-id removal to git")?;
                        }
                    }
                } else {
                    return Err(
                        StoreError::NoGpgId("Cannot clear gpg-ids as this would leave the store without any gpg-ids".to_string())
                    );
                }
            }

            Ok(())
        };

        let steps = (|| {
            write_gpg_ids(&gpg_ids, self.store)?;

            let joined_ids = gpg_ids.iter()
                .map(GpgKeyId::id)
                .collect::<Vec<_>>()
                .join(", ");
            let passwords = self.store
                .show(self.path(), TraversalOrder::PreOrder)?
                .filter_map(Entry::password)
                .collect::<Vec<_>>();
            for password in &passwords {
                password.decrypt()?
                    .save(
                        Some(format!(
                            "Reencrypt '{}' as gpg-ids changed to {}.",
                            password.path()
                                .strip_prefix(&root)
                                .unwrap()
                                .with_extension("")
                                .display(),
                            joined_ids,
                        )),
                        self.store,
                    )?;
            }

            Ok(())
        })();

        if steps.is_err() {
            // TODO: Reencryption might have already reencrypted some passwords.
            //       What to do now?
            write_gpg_ids(&old_gpg_ids, self.store)?;
        }

        steps
    }

    pub fn gpg_ids(&self) -> Result<Vec<GpgKeyId>, StoreError> {
        get_gpg_ids_for_path(self.path())
    }

    pub fn add_gpg_id(&mut self, gpg_key: GpgKeyId) -> Result<(), StoreError> {
        let mut gpg_ids = self.gpg_ids()?;
        gpg_ids.push(gpg_key);
        self.set_new_gpg_ids_and_reencrypt_passwords(gpg_ids)
    }

    pub fn remove_gpg_id(&mut self, gpg_key: GpgKeyId) -> Result<(), StoreError> {
        let mut gpg_ids = self.gpg_ids()?;
        gpg_ids.retain(|key| *key != gpg_key);
        self.set_new_gpg_ids_and_reencrypt_passwords(gpg_ids)
    }

    pub fn clear_gpg_ids(&mut self) -> Result<(), StoreError> {
        self.set_new_gpg_ids_and_reencrypt_passwords(Vec::new())
    }

    pub fn set_gpg_ids(&mut self, gpg_ids: Vec<GpgKeyId>) -> Result<(), StoreError> {
        self.set_new_gpg_ids_and_reencrypt_passwords(gpg_ids)
    }

    pub fn parent(self) -> Option<MutDirectory<'a>> {
        // consume self here, to avoid a parent directory being removed and
        // having references to node_ids of child directory entries
        let parent_id = self.store.tree.ancestor_ids(&self.node_id).ok()?.next()?;

        Some(MutDirectory::new(parent_id.clone(), self.store))
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
        self.store
            .tree
            .remove_node(self.node_id, RemoveBehavior::DropChildren)
            .expect("Could not remove password from internal tree structure");

        let root = self.store.location().to_owned();
        if let Some(git) = self.store.git() {
            git.add(&[&path])
                .with_store_error("failed to add removal to git")?;
            git.commit(&format!(
                "Remove '{}' from store.",
                path.strip_prefix(root)
                    .unwrap()
                    .with_extension("")
                    .display(),
            ))
            .with_store_error("failed to commit removal to git")?;
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
        Directory::new(
            self.name().into(),
            self.path().into(),
            self.store.location().into(),
            self.node_id,
        )
    }
}
