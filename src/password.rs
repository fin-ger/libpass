use anyhow::{Result, Context as AnyhowContext};
use gpgme::{Context, Protocol};

use std::path::Path;
use std::fs::File;
use std::collections::HashMap;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "pass.pest"]
pub struct PasswordParser;

pub struct DecryptedPassword {
    password: String,
    comments: Vec<String>,
    entries: HashMap<String, String>,
}

impl DecryptedPassword {
    fn new(path: &Path) -> Result<Self> {
        let mut pw = File::open(path)
            .context(format!("Cannot open password file: {}", path.display()))?;
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)
            .context("Cannot create GPG context")?;
        let mut content = Vec::new();
        // TODO: Add password provider
        ctx.decrypt(&mut pw, &mut content)
            .context(format!("Could not decrypt password {}", path.display()))?;

        let content = String::from_utf8_lossy(&content);
        let content = PasswordParser::parse(Rule::content, &content)
            .context(format!("Could not parse password content {}", path.display()))?
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

    pub fn decrypt(&self) -> Result<DecryptedPassword> {
        DecryptedPassword::new(self.path)
    }
}

