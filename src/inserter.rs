use id_tree::NodeId;
use std::path::PathBuf;

use crate::{Directory, Password, Store, StoreError};

#[cfg(feature = "passphrase-utils")]
use crate::passphrase_utils::{AnalyzedPassphrase, PassphraseGenerator};

pub struct PasswordInserter {
    pub(crate) parent: NodeId,
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) lines: Vec<String>,
    pub(crate) changes: Vec<String>,
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
            changes: Vec::new(),
        }
    }

    pub fn passphrase<P: Into<String>>(&mut self, passphrase: P) -> &mut Self {
        if let Some(pw) = self.lines.get_mut(0) {
            *pw = passphrase.into();
        } else {
            self.lines.push(passphrase.into());
        }
        self.changes.push("Add given passphrase to password".into());

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

    #[cfg(feature = "passphrase-utils")]
    pub fn generator(&mut self) -> PassphraseGenerator<&mut Self> {
        PassphraseGenerator::new(move |passphrase| {
            if let Some(pw) = self.lines.get_mut(0) {
                *pw = passphrase.into();
            } else {
                self.lines.push(passphrase.into());
            }
            self.changes.push("Add generated passphrase to password".into());

            Ok(self)
        })
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn analyze_passphrase(&self) -> Option<AnalyzedPassphrase> {
        let passphrase = self.lines.get(0)?;
        Some(AnalyzedPassphrase::new(passphrase))
    }

    pub fn insert(&self, store: &mut Store) -> Result<Password, StoreError> {
        store.insert_password(self)
    }
}

pub struct DirectoryInserter {
    pub(crate) parent: NodeId,
    pub(crate) path: PathBuf,
    pub(crate) name: String,
}

impl DirectoryInserter {
    pub(crate) fn new(parent: NodeId, path: PathBuf, name: String) -> Self {
        Self { parent, name, path }
    }

    pub fn insert(&self, store: &mut Store) -> Result<Directory, StoreError> {
        store.insert_directory(self)
    }
}
