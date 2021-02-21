use id_tree::NodeId;
use std::{collections::HashMap, path::PathBuf};

use crate::{Password, Position, StoreError};

pub struct PasswordInserter {
    pub(crate) parent: NodeId,
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) passphrase: Option<String>,
    pub(crate) comments: Vec<(Position, String)>,
    pub(crate) entries: HashMap<String, (Position, String)>,
    pub(crate) back: Position,
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
            passphrase: None,
            comments: Vec::new(),
            entries: HashMap::new(),
            back: 0,
        }
    }

    pub fn passphrase<P: Into<String>>(&mut self, passphrase: P) -> &mut Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    pub fn comment<C: Into<String>>(&mut self, comment: C) -> &mut Self {
        self.comments.push((self.back, comment.into()));
        self.back += 1;
        self
    }

    pub fn comments(&mut self, comments: Vec<(Position, String)>) -> &mut Self {
        self.comments = comments;
        self.back = self.comments.len() + self.entries.len();
        self
    }

    pub fn entry<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> &mut Self {
        self.entries.insert(key.into(), (self.back, value.into()));
        self.back += 1;
        self
    }

    pub fn entries(&mut self, entries: HashMap<String, (Position, String)>) -> &mut Self {
        self.entries = entries;
        self.back = self.comments.len() + self.entries.len();
        self
    }

    pub fn insert(&self, store: &mut crate::Store) -> Result<Password, StoreError> {
        store.insert_parsed_password(self)
    }
}
