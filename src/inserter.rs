use id_tree::Tree;
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

use crate::{Directory, PassNode, Password};

#[derive(Error, Debug)]
pub enum InsertionError {}

pub struct PasswordInserter<'a> {
    tree: &'a mut Tree<PassNode>,
    path: PathBuf,
    name: String,
    passphrase: Option<String>,
    comments: Vec<String>,
    entries: HashMap<String, String>,
}

impl<'a> PasswordInserter<'a> {
    pub(crate) fn new(tree: &'a mut Tree<PassNode>, path: PathBuf, name: String) -> Self {
        Self {
            tree,
            path,
            name,
            passphrase: None,
            comments: Vec::new(),
            entries: HashMap::new(),
        }
    }

    pub fn passphrase<P: Into<String>>(self, passphrase: P) -> Self {
        Self {
            passphrase: Some(passphrase.into()),
            ..self
        }
    }

    pub fn comment<C: Into<String>>(mut self, comment: C) -> Self {
        self.comments.push(comment.into());
        self
    }

    pub fn comments(self, comments: Vec<String>) -> Self {
        Self { comments, ..self }
    }

    pub fn entry<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.entries.insert(key.into(), value.into());
        self
    }

    pub fn entries(self, entries: HashMap<String, String>) -> Self {
        Self { entries, ..self }
    }

    pub fn insert(self) -> Result<Password, InsertionError> {
        

        //self.tree.insert(node, behavior)
        // TODO: insert into Tree<PassNode>
        todo!();
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

    pub fn insert(self) -> Result<Directory, InsertionError> {
        // TODO: insert into Tree<PassNode>
        todo!();
    }
}
