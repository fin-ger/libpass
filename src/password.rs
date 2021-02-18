use std::fs::File;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use gpgme::{Context, Protocol};
use id_tree::{NodeId, Tree};
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::{PassNode, Store};

#[derive(Parser, Debug)]
#[grammar = "pass.pest"]
pub struct PasswordParser;

pub struct DecryptedPassword {
    passphrase: String,
    comments: Vec<String>,
    entries: HashMap<String, String>,
}

impl DecryptedPassword {
    fn new(path: &Path) -> Result<Self, StoreReadError> {
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

        for record in content.into_inner() {
            println!("{}", record);
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
                    entries.insert(key, value);
                }
                Rule::comment => {
                    comments.push(record.as_str().to_owned());
                }
                _ => unreachable!(),
            }
        }

        Ok(Self {
            passphrase,
            comments,
            entries,
        })
    }

    pub fn passphrase<'a>(&'a self) -> &'a str {
        &self.passphrase
    }

    pub fn comments<'a>(&'a self) -> &'a Vec<String> {
        &self.comments
    }

    pub fn entry<'a>(&'a self, key: &str) -> Option<&'a String> {
        self.entries.get(key)
    }

    pub fn all_entries<'a>(&'a self) -> &'a HashMap<String, String> {
        &self.entries
    }
}

pub struct Password<'a> {
    name: &'a str,
    path: &'a Path,
    node_id: NodeId,
}

impl<'a> Password<'a> {
    pub(crate) fn new(name: &'a str, path: &'a Path, node: NodeId) -> Self {
        Self {
            name,
            path,
            node_id: node,
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn path(&self) -> &'a Path {
        self.path
    }

    pub(crate) fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn decrypt(&self) -> Result<DecryptedPassword, StoreReadError> {
        DecryptedPassword::new(self.path)
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
        DecryptedPassword::new(&self.path)
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
