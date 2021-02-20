use std::{fmt, fs::{File, OpenOptions}, io::Write};
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use gpgme::{Context, Protocol};
use id_tree::{NodeId, Tree};
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::{IntoStoreError, PassNode, Store, StoreError};

pub type Position = usize;

#[derive(Parser, Debug)]
#[grammar = "pass.pest"]
pub struct PasswordParser;

pub struct DecryptedPassword {
    passphrase: String,
    comments: Vec<(Position, String)>,
    entries: HashMap<String, (Position, String)>,
    path: PathBuf,
    back: Position,
}

impl fmt::Display for DecryptedPassword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n", self.passphrase)?;
        let comments = self.comments.clone().into_iter();
        let entries = self.entries.iter().map(|(key, (position, value))| {
            (*position, format!("{}: {}", key, value))
        });
        let mut content = comments.chain(entries).collect::<Vec<_>>();
        content.sort_by_key(|(position, _comment)| *position);
        for (_position, line) in content {
            write!(f, "{}\n", line)?;
        }
        Ok(())
    }
}

impl DecryptedPassword {
    pub(crate) fn from_path(path: &Path) -> Result<Self, StoreReadError> {
        let mut pw = File::open(path).with_store_read_error(path)?;
        let mut ctx = Context::from_protocol(Protocol::OpenPgp).with_store_read_error(path)?;
        let mut content = Vec::new();
        // TODO: Add passphrase provider
        ctx.decrypt(&mut pw, &mut content)
            .with_store_read_error(path)?;

        let content = String::from_utf8_lossy(&content);
        let content = PasswordParser::parse(Rule::content, &content)
            .with_store_read_error(path)?
            .next()
            .unwrap(); // unwrap 'content' rule which is always available

        let mut passphrase = String::new();
        let mut comments = Vec::new();
        let mut entries = HashMap::new();

        let mut position = 0;
        for record in content.into_inner() {
            match record.as_rule() {
                Rule::password => {
                    passphrase = record.as_str().to_owned();
                }
                Rule::entry => {
                    let mut key = String::new();
                    let mut value = String::new();
                    for record in record.into_inner() {
                        match record.as_rule() {
                            Rule::key => {
                                key = record.as_str().to_owned();
                            }
                            Rule::value => {
                                value = record.as_str().to_owned();
                            }
                            _ => unreachable!(),
                        }
                    }
                    entries.insert(key, (position, value));
                }
                Rule::comment => {
                    comments.push((position, record.as_str().to_owned()));
                }
                _ => unreachable!(),
            }
            position += 1;
        }

        Ok(Self {
            passphrase,
            comments,
            entries,
            path: path.to_owned(),
            back: position,
        })
    }

    pub(crate) fn create_and_write(
        passphrase: String,
        comments: Vec<(Position, String)>,
        entries: HashMap<String, (Position, String)>,
        back: Position,
        path: &Path,
    ) -> Result<Self, StoreError> {
        let me = Self {
            passphrase,
            comments,
            entries,
            back,
            path: path.to_owned(),
        };
        me.save()?;
        Ok(me)
    }

    fn save(&self) -> Result<(), StoreError> {
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)
            .with_store_error()?;
        f.write_all(format!("{}", self).as_bytes()).with_store_error()?;
        f.flush().with_store_error()
    }

    pub fn passphrase(&self) -> &str {
        &self.passphrase
    }

    pub fn set_passphrase<P: Into<String>>(&mut self, passphrase: P) -> Result<(), StoreError> {
        let old_passphrase = std::mem::replace(&mut self.passphrase, passphrase.into());
        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.passphrase = old_passphrase;
                Err(err)
            },
        }
    }

    pub fn comments(&self) -> impl Iterator<Item = (Position, &str)> {
        self.comments.iter().map(|(position, comment)| (*position, comment.as_str()))
    }

    pub fn set_comments<C: Into<Vec<(Position, String)>>>(
        &mut self,
        comments: C,
    ) -> Result<(), StoreError> {
        let old_comments = std::mem::replace(&mut self.comments, comments.into());
        let old_back = std::mem::replace(&mut self.back, self.entries.len() + self.comments.len());
        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.comments = old_comments;
                self.back = old_back;
                Err(err)
            },
        }
    }

    pub fn insert_comment<C: Into<String>>(&mut self, position: Position, comment: C) -> Result<(), StoreError> {
        let comment = comment.into();
        let old_comments = std::mem::replace(&mut self.comments, Vec::new());
        let mut inserted = false;
        for (pos, com) in old_comments.clone() {
            if pos < position {
                self.comments.push((pos, com));
            } else if inserted {
                self.comments.push((pos + 1, com));
            } else {
                self.comments.push((pos, comment.clone()));
                self.comments.push((pos + 1, com));
                inserted = true;
            }
        }

        let new_entries = self.entries.clone().into_iter().map(|(key, (pos, value))| {
            if pos < position {
                (key, (pos, value))
            } else {
                (key, (pos + 1, value))
            }
        }).collect();
        let old_entries = std::mem::replace(&mut self.entries, new_entries);
        let old_back = std::mem::replace(&mut self.back, self.entries.len() + self.comments.len());

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.comments = old_comments;
                self.entries = old_entries;
                self.back = old_back;
                Err(err)
            },
        }
    }

    pub fn prepend_comment<C: Into<String>>(&mut self, comment: C) -> Result<(), StoreError> {
        self.insert_comment(0, comment)
    }

    pub fn append_comment<C: Into<String>>(&mut self, comment: C) -> Result<(), StoreError> {
        self.insert_comment(self.back, comment)
    }

    pub fn set_entries<E: Into<HashMap<String, (Position, String)>>>(
        &mut self,
        entries: E,
    ) -> Result<(), StoreError> {
        let old_entries = std::mem::replace(&mut self.entries, entries.into());
        let old_back = std::mem::replace(&mut self.back, self.entries.len() + self.comments.len());
        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.entries = old_entries;
                self.back = old_back;
                Err(err)
            },
        }
    }

    pub fn insert_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        position: Position,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        let key = key.into();
        let value = value.into();
        let old_entries = std::mem::replace(&mut self.entries, HashMap::new());
        let mut inserted = false;
        for (k, (pos, v)) in old_entries.clone() {
            if pos < position {
                self.entries.insert(k, (pos, v));
            } else if inserted {
                self.entries.insert(k, (pos + 1, v));
            } else {
                self.entries.insert(key.clone(), (pos, value.clone()));
                self.entries.insert(k, (pos + 1, v));
                inserted = true;
            }
        }

        let new_comments = self.comments.clone().into_iter().map(|(pos, com)| {
            if pos < position {
                (pos, com)
            } else {
                (pos + 1, com)
            }
        }).collect();
        let old_comments = std::mem::replace(&mut self.comments, new_comments);
        let old_back = std::mem::replace(&mut self.back, self.entries.len() + self.comments.len());

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.comments = old_comments;
                self.entries = old_entries;
                self.back = old_back;
                Err(err)
            },
        }
    }

    pub fn prepend_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        self.insert_entry(0, key, value)
    }

    pub fn append_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        self.insert_entry(self.back, key, value)
    }

    pub fn entry(&self, key: &str) -> Option<(Position, &str)> {
        self.entries.get(key).map(|(position, value)| (*position, value.as_str()))
    }

    pub fn all_entries(&self) -> impl Iterator<Item = (&str, (Position, &str))> {
        self.entries.iter().map(|(key, (position, value))| (key.as_str(), (*position, value.as_str())))
    }
}

pub struct Password {
    name: String,
    path: PathBuf,
    node_id: NodeId,
}

impl Password {
    pub(crate) fn new(name: String, path: PathBuf, node_id: NodeId) -> Self {
        Self {
            name,
            path,
            node_id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn decrypt(&self) -> Result<DecryptedPassword, StoreReadError> {
        DecryptedPassword::from_path(&self.path)
    }

    pub fn make_mut(self, store: &mut Store) -> MutPassword {
        store.mut_password(self)
    }
}

pub struct MutPassword<'a> {
    name: String,
    path: PathBuf,
    tree: &'a mut Tree<PassNode>,
    node_id: NodeId,
}

impl<'a> MutPassword<'a> {
    pub(crate) fn new(
        name: String,
        path: PathBuf,
        tree: &'a mut Tree<PassNode>,
        node: NodeId,
    ) -> Self {
        Self {
            name,
            path,
            tree,
            node_id: node,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn decrypt(&self) -> Result<DecryptedPassword, StoreReadError> {
        DecryptedPassword::from_path(&self.path)
    }
}

#[derive(Error, Debug)]
pub enum StoreReadError {
    #[error("Cannot retrieve password or directory name from `{0}`")]
    Name(String),
    #[error("Cannot open password file or directory `{0}`")]
    Open(String, #[source] std::io::Error),
    #[error("Failed to decrypt password `{0}`")]
    Decrypt(String, #[source] gpgme::Error),
    #[error("Failed to parse password content of `{0}`")]
    Parse(String, #[source] pest::error::Error<Rule>),
}

impl StoreReadError {
    pub(crate) fn name_error(path: &Path) -> StoreReadError {
        Self::Name(format!("{}", path.display()))
    }
}

pub(crate) trait IntoStoreReadError<T> {
    fn with_store_read_error(self: Self, path: &Path) -> Result<T, StoreReadError>;
}

impl<T> IntoStoreReadError<T> for Result<T, std::io::Error> {
    fn with_store_read_error(self: Self, path: &Path) -> Result<T, StoreReadError> {
        self.map_err(|err| StoreReadError::Open(format!("{}", path.display()), err))
    }
}

impl<T> IntoStoreReadError<T> for Result<T, gpgme::Error> {
    fn with_store_read_error(self: Self, path: &Path) -> Result<T, StoreReadError> {
        self.map_err(|err| StoreReadError::Decrypt(format!("{}", path.display()), err))
    }
}

impl<T> IntoStoreReadError<T> for Result<T, pest::error::Error<Rule>> {
    fn with_store_read_error(self: Self, path: &Path) -> Result<T, StoreReadError> {
        self.map_err(|err| StoreReadError::Parse(format!("{}", path.display()), err))
    }
}
