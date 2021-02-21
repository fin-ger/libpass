use pest::Parser;
use pest_derive::Parser;
use gpgme::{Context, Protocol};
use std::{
    collections::HashMap,
    fmt,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{search_gpg_ids, IntoStoreError, Position, StoreError};

#[cfg(feature = "passphrase-utils")]
use crate::passphrase_utils::{AnalyzedPassphrase, PassphraseGenerator};

#[derive(Parser, Debug)]
#[grammar = "parsed/pass.pest"]
struct PasswordParser;

pub struct DecryptedPassword {
    passphrase: Option<String>,
    comments: Vec<(Position, String)>,
    entries: HashMap<String, (Position, String)>,
    path: PathBuf,
    back: Position,
}

impl fmt::Display for DecryptedPassword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(passphrase) = &self.passphrase {
            write!(f, "{}\n", passphrase)?;
        }
        let comments = self.comments.clone().into_iter();
        let entries = self
            .entries
            .iter()
            .map(|(key, (position, value))| (*position, format!("{}: {}", key, value)));
        let mut content = comments.chain(entries).collect::<Vec<_>>();
        content.sort_by_key(|(position, _comment)| *position);
        for (_position, line) in content {
            write!(f, "{}\n", line)?;
        }
        Ok(())
    }
}

impl DecryptedPassword {
    pub(crate) fn from_lines(lines: Vec<String>, path: PathBuf) -> Result<Self, StoreError> {
        let content = lines.join("\n");
        let content = PasswordParser::parse(Rule::content, &content)
            .map_err(|err| StoreError::Parse(path.display().to_string(), Box::new(err)))?
            .next()
            .unwrap(); // unwrap 'content' rule which is always available

        let mut passphrase = None;
        let mut comments = Vec::new();
        let mut entries = HashMap::new();

        let mut position = 0;
        for record in content.into_inner() {
            match record.as_rule() {
                Rule::password => {
                    passphrase = Some(record.as_str().to_owned());
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
        passphrase: Option<String>,
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
            .with_store_error(self.path.display().to_string())?;
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)
            .with_store_error(self.path.display().to_string())?;
        let content = format!("{}", self);
        let mut encrypted = Vec::new();
        let gpg_ids = search_gpg_ids(&self.path, &mut ctx);
        ctx.encrypt(gpg_ids.iter(), content, &mut encrypted)
            .with_store_error(self.path.display().to_string())?;

        f.write_all(&encrypted)
            .with_store_error(self.path.display().to_string())?;
        f.flush().with_store_error(self.path.display().to_string())
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn generator(&mut self) -> PassphraseGenerator {
        PassphraseGenerator::new(move |passphrase| self.set_passphrase(passphrase))
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn analyze_passphrase(&self) -> Option<AnalyzedPassphrase> {
        let passphrase = self.passphrase()?;

        Some(AnalyzedPassphrase::new(passphrase))
    }

    pub fn passphrase(&self) -> Option<&str> {
        self.passphrase.as_ref().map(|p| p.as_str())
    }

    pub fn set_passphrase<P: Into<String>>(&mut self, passphrase: P) -> Result<(), StoreError> {
        let old_passphrase = self.passphrase.replace(passphrase.into());
        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.passphrase = old_passphrase;
                Err(err)
            }
        }
    }

    pub fn comments(&self) -> impl Iterator<Item = (Position, &str)> {
        self.comments
            .iter()
            .map(|(position, comment)| (*position, comment.as_str()))
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
            }
        }
    }

    pub fn insert_comment<C: Into<String>>(
        &mut self,
        position: Position,
        comment: C,
    ) -> Result<(), StoreError> {
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

        let new_entries = self
            .entries
            .clone()
            .into_iter()
            .map(|(key, (pos, value))| {
                if pos < position {
                    (key, (pos, value))
                } else {
                    (key, (pos + 1, value))
                }
            })
            .collect();
        let old_entries = std::mem::replace(&mut self.entries, new_entries);
        let old_back = std::mem::replace(&mut self.back, self.entries.len() + self.comments.len());

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.comments = old_comments;
                self.entries = old_entries;
                self.back = old_back;
                Err(err)
            }
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
            }
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

        let new_comments = self
            .comments
            .clone()
            .into_iter()
            .map(|(pos, com)| {
                if pos < position {
                    (pos, com)
                } else {
                    (pos + 1, com)
                }
            })
            .collect();
        let old_comments = std::mem::replace(&mut self.comments, new_comments);
        let old_back = std::mem::replace(&mut self.back, self.entries.len() + self.comments.len());

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.comments = old_comments;
                self.entries = old_entries;
                self.back = old_back;
                Err(err)
            }
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
        self.entries
            .get(key)
            .map(|(position, value)| (*position, value.as_str()))
    }

    pub fn all_entries(&self) -> impl Iterator<Item = (&str, (Position, &str))> {
        self.entries
            .iter()
            .map(|(key, (position, value))| (key.as_str(), (*position, value.as_str())))
    }
}
