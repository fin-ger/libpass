use std::path::Path;
use std::fs::File;
use std::collections::HashMap;

use thiserror::Error;
use gpgme::{Context, Protocol};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser, Debug)]
#[grammar = "pass.pest"]
pub struct PasswordParser;

pub struct DecryptedPassword {
    password: String,
    comments: Vec<String>,
    entries: HashMap<String, String>,
}

impl DecryptedPassword {
    fn new(path: &Path) -> Result<Self, StoreReadError> {
        let mut pw = File::open(path)
            .with_store_read_error(path)?;
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)
            .with_store_read_error(path)?;
        let mut content = Vec::new();
        // TODO: Add passphrase provider
        ctx.decrypt(&mut pw, &mut content)
            .with_store_read_error(path)?;

        let content = String::from_utf8_lossy(&content);
        let content = PasswordParser::parse(Rule::content, &content)
            .with_store_read_error(path)?
            .next().unwrap(); // unwrap 'content' rule which is always available

        let mut password = String::new();
        let mut comments = Vec::new();
        let mut entries = HashMap::new();

        for record in content.into_inner() {
            println!("{}", record);
            match record.as_rule() {
                Rule::password => {
                    password = record.as_str().to_owned();
                },
                Rule::entry => {
                    let mut key = String::new();
                    let mut value = String::new();
                    for record in record.into_inner() {
                        match record.as_rule() {
                            Rule::key => {
                                key = record.as_str().to_owned();
                            },
                            Rule::value => {
                                value = record.as_str().to_owned();
                            },
                            _ => unreachable!(),
                        }
                    }
                    entries.insert(key, value);
                },
                Rule::comment => {
                    comments.push(record.as_str().to_owned());
                },
                _ => unreachable!(),
            }
        }

        Ok(Self {
            password,
            comments,
            entries,
        })
    }

    pub fn password<'a>(&'a self) -> &'a str {
        &self.password
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
}

impl<'a> Password<'a> {
    pub(crate) fn new(name: &'a str, path: &'a Path) -> Self {
        Self {
            name,
            path,
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn path(&self) -> &'a Path {
        self.path
    }

    pub fn decrypt(&self) -> Result<DecryptedPassword, StoreReadError> {
        DecryptedPassword::new(self.path)
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
        self.map_err(|err| {
            StoreReadError::Open(format!("{}", path.display()), err)
        })
    }
}

impl<T> IntoStoreReadError<T> for Result<T, gpgme::Error> {
    fn with_store_read_error(self: Self, path: &Path) -> Result<T, StoreReadError> {
        self.map_err(|err| {
            StoreReadError::Decrypt(format!("{}", path.display()), err)
        })
    }
}

impl<T> IntoStoreReadError<T> for Result<T, pest::error::Error<Rule>> {
    fn with_store_read_error(self: Self, path: &Path) -> Result<T, StoreReadError> {
        self.map_err(|err| {
            StoreReadError::Parse(format!("{}", path.display()), err)
        })
    }
}
