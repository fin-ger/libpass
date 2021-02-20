use id_tree::{InsertBehavior, Node, NodeId, Tree};
use std::{collections::HashMap, path::PathBuf};

use crate::{DecryptedPassword, Directory, PassNode, Password, Position, StoreError};

pub struct PasswordInserter<'a> {
    tree: &'a mut Tree<PassNode>,
    parent: NodeId,
    path: PathBuf,
    name: String,
    passphrase: Option<String>,
    comments: Vec<(Position, String)>,
    entries: HashMap<String, (Position, String)>,
    back: Position,
}

impl<'a> PasswordInserter<'a> {
    pub(crate) fn new(
        tree: &'a mut Tree<PassNode>,
        parent: NodeId,
        path: PathBuf,
        name: String,
    ) -> Self {
        Self {
            tree,
            parent,
            path,
            name,
            passphrase: None,
            comments: Vec::new(),
            entries: HashMap::new(),
            back: 0,
        }
    }

    pub fn passphrase<P: Into<String>>(self, passphrase: P) -> Self {
        Self {
            passphrase: Some(passphrase.into()),
            ..self
        }
    }

    pub fn comment<C: Into<String>>(mut self, comment: C) -> Self {
        self.comments.push((self.back, comment.into()));
        self.back += 1;
        self
    }

    pub fn comments(self, comments: Vec<(Position, String)>) -> Self {
        Self {
            comments,
            back: self.comments.len() + self.entries.len(),
            ..self
        }
    }

    pub fn entry<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.entries.insert(key.into(), (self.back, value.into()));
        self.back += 1;
        self
    }

    pub fn entries(self, entries: HashMap<String, (Position, String)>) -> Self {
        Self {
            entries,
            back: self.comments.len() + self.entries.len(),
            ..self
        }
    }

    pub fn insert(self) -> Result<Password, StoreError> {
        DecryptedPassword::create_and_write(
            self.passphrase.unwrap_or(String::new()),
            self.comments,
            self.entries,
            self.back,
            &self.path,
        )?;

        let node = Node::new(PassNode::Password {
            name: self.name.clone(),
            path: self.path.clone(),
        });
        let node_id = self
            .tree
            .insert(node, InsertBehavior::UnderNode(&self.parent))
            .expect("Parent of inserted password does not exist in internal tree");

        Ok(Password::new(self.name, self.path, node_id))
    }
}

pub struct DirectoryInserter<'a> {
    tree: &'a mut Tree<PassNode>,
    path: PathBuf,
    name: String,
}

impl<'a> DirectoryInserter<'a> {
    pub(crate) fn new(tree: &'a mut Tree<PassNode>, path: PathBuf, name: String) -> Self {
        Self { tree, name, path }
    }

    pub fn insert(self) -> Result<Directory, StoreError> {
        // TODO: insert into Tree<PassNode>
        todo!();
    }
}
