use id_tree::NodeId;
use std::path::PathBuf;

use crate::{Directory, Password, Store, StoreError};

pub struct PasswordInserter {
    pub(crate) parent: NodeId,
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) lines: Vec<String>,
}

impl PasswordInserter {
    pub(crate) fn new(
        parent: NodeId,
        path: PathBuf,
        name: String,
    ) -> Self {
        Self {
            parent,
            path,
            name,
            lines: Vec::new(),
        }
    }

    pub fn passphrase<P: Into<String>>(&mut self, passphrase: P) -> &mut Self {
        if let Some(pw) = self.lines.get_mut(0) {
            *pw = passphrase.into();
        } else {
            self.lines.push(passphrase.into());
        }

        self
    }

    pub fn line<L: Into<String>>(&mut self, comment: L) -> &mut Self {
        self.lines.push(comment.into());
        self
    }

    pub fn lines(&mut self, lines: Vec<String>) -> &mut Self {
        self.lines = lines;
        self
    }

    pub fn insert(&self, store: &mut Store) -> Result<Password, StoreError> {
        store.insert_password(self)
    }
}

pub struct DirectoryInserter {
    pub path: PathBuf,
    pub name: String,
}

impl DirectoryInserter {
    pub(crate) fn new(path: PathBuf, name: String) -> Self {
        Self { name, path }
    }

    pub fn insert(&self, store: &mut Store) -> Result<Directory, StoreError> {
        store.insert_directory(self)
    }
}
