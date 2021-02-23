use id_tree::NodeId;
use std::path::PathBuf;

use crate::{Password, StoreError};

#[cfg(feature = "passphrase-utils")]
use crate::passphrase_utils::{AnalyzedPassphrase, PassphraseGenerator};

use crate::parsed::PasswordLine;

pub struct PasswordInserter {
    pub(crate) parent: NodeId,
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) passphrase: Option<String>,
    pub(crate) lines: Vec<PasswordLine>,
}

impl PasswordInserter {
    pub(crate) fn new(parent: NodeId, path: PathBuf, name: String) -> Self {
        Self {
            parent,
            path,
            name,
            passphrase: None,
            lines: Vec::new(),
        }
    }

    pub fn passphrase<P: Into<String>>(&mut self, passphrase: P) -> &mut Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    pub fn lines(&mut self, lines: Vec<PasswordLine>) -> &mut Self {
        self.lines = lines;
        self
    }

    pub fn comment<C: Into<String>>(&mut self, comment: C) -> &mut Self {
        self.lines.push(PasswordLine::Comment(comment.into()));
        self
    }

    pub fn entry<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.lines
            .push(PasswordLine::Entry(key.into(), value.into()));
        self
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn generator(&mut self) -> PassphraseGenerator<&mut Self> {
        PassphraseGenerator::new(move |passphrase| Ok(self.passphrase(passphrase)))
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn analyze_passphrase(&self) -> Option<AnalyzedPassphrase> {
        let passphrase = self.passphrase.as_ref()?;
        Some(AnalyzedPassphrase::new(passphrase))
    }

    pub fn insert(&self, store: &mut crate::Store) -> Result<Password, StoreError> {
        store.insert_parsed_password(self)
    }
}
