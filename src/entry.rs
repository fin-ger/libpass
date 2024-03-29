use id_tree::NodeId;
use id_tree::RemoveBehavior;

use std::path::Path;
use std::{fmt, path::PathBuf};
use std::{fs, io};

use crate::{
    Directory, EntryKind, IntoStoreError, MutDirectory, MutPassword, PassNode, Password, Store, StoreError,
};

pub struct Entry {
    data: PassNode,
    node_id: NodeId,
    root: PathBuf,
}

impl Entry {
    pub(crate) fn new(node_id: NodeId, data: PassNode, root: PathBuf) -> Self {
        Self {
            data,
            node_id,
            root,
        }
    }

    pub fn name(&self) -> &str {
        self.data.name()
    }

    pub fn path(&self) -> &Path {
        self.data.path()
    }

    pub fn kind(&self) -> EntryKind {
        self.data.kind()
    }

    pub(crate) fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn password(self) -> Option<Password> {
        if let PassNode::Password { name, path } = self.data {
            Some(Password::new(name, path, self.root, self.node_id))
        } else {
            None
        }
    }

    pub fn directory(self) -> Option<Directory> {
        if let PassNode::Directory { name, path } = self.data {
            Some(Directory::new(name, path, self.root, self.node_id))
        } else {
            None
        }
    }

    pub fn make_mut(self, store: &mut Store) -> MutEntry {
        store.mut_entry(self)
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind().fmt(f);
        f.debug_struct("Entry")
            .field("kind", &kind)
            .field("name", &self.name().to_string())
            .field("path", &self.path().display().to_string())
            .finish()
    }
}

#[derive(Debug, PartialEq)]
pub enum Traversal {
    None,
    Recursive,
}

pub struct MutEntry<'a> {
    store: &'a mut Store,
    node_id: NodeId,
}

impl<'a> MutEntry<'a> {
    pub(crate) fn new(node_id: NodeId, store: &'a mut Store) -> Self {
        Self { store, node_id }
    }

    fn data_mut(&mut self) -> &mut PassNode {
        self.store.tree.get_mut(&self.node_id).unwrap().data_mut()
    }

    fn data(&self) -> &PassNode {
        self.store.tree.get(&self.node_id).unwrap().data()
    }

    pub fn name(&self) -> &str {
        self.data().name()
    }

    pub fn path(&self) -> &Path {
        self.data().path()
    }

    pub fn kind(&self) -> EntryKind {
        self.data().kind()
    }

    pub fn remove(self, traversal: Traversal) -> Result<(), StoreError> {
        match self.kind() {
            EntryKind::Directory => self.mut_directory().unwrap().remove(traversal),
            _ => {
                let path = self.path().to_owned();

                fs::remove_file(&path).with_store_error("Could not remove password")?;
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
            },
        }
    }

    pub fn rename<N: Into<String>>(&mut self, new_name: N) -> Result<(), StoreError> {
        let old_path = self.path().to_owned();
        if old_path == *self.store.location() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Cannot rename store's root directory",
            ))
            .with_store_error("Attempted to rename store's root directory")?;
        }
        let mut new_path = old_path.with_file_name(new_name.into());
        if self.kind() == EntryKind::Password {
            new_path = new_path.with_extension("gpg");
        }
        fs::rename(&old_path, &new_path).with_store_error("Failed to rename store entry")?;

        let (name, path) = match self.data_mut() {
            PassNode::Password { name, path } => (name, path),
            PassNode::Directory { name, path } => (name, path),
            PassNode::NormalFile { name, path } => (name, path),
        };
        *path = new_path.clone();
        *name = path.file_stem().unwrap().to_string_lossy().to_string();

        let root = self.store.location().to_owned();
        if let Some(git) = self.store.git() {
            git.add(&[&old_path, &new_path])
                .with_store_error("failed to add rename to git")?;
            git.commit(&format!(
                "Rename '{}' to '{}'.",
                old_path
                    .strip_prefix(&root)
                    .unwrap()
                    .with_extension("")
                    .display(),
                new_path
                    .strip_prefix(&root)
                    .unwrap()
                    .with_extension("")
                    .display(),
            ))
            .with_store_error("failed to commit rename to git")?;
        }

        Ok(())
    }

    pub fn move_to(&mut self, _directory: &Directory) {}

    pub fn copy_to(&mut self, _directory: &Directory) {}

    pub fn mut_password(self) -> Option<MutPassword<'a>> {
        if self.kind() == EntryKind::Password {
            Some(MutPassword::new(self.node_id, self.store))
        } else {
            None
        }
    }

    pub fn mut_directory(self) -> Option<MutDirectory<'a>> {
        if self.kind() == EntryKind::Directory {
            Some(MutDirectory::new(self.node_id, self.store))
        } else {
            None
        }
    }
}

impl<'a> fmt::Debug for MutEntry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind().fmt(f);
        f.debug_struct("MutEntry")
            .field("kind", &kind)
            .field("name", &self.name().to_string())
            .field("path", &self.path().display().to_string())
            .finish()
    }
}
