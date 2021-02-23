use gpgme::{Context, Protocol};
use pest::Parser;
use pest_derive::Parser;
use std::{
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

#[derive(Debug, Clone)]
pub enum PasswordLine {
    Comment(String),
    Entry(String, String),
}

impl fmt::Display for PasswordLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PasswordLine::Comment(comment) => write!(f, "{}", comment),
            PasswordLine::Entry(key, value) => write!(f, "{}: {}", key, value),
        }
    }
}

pub struct DecryptedPassword {
    passphrase: Option<String>,
    lines: Vec<PasswordLine>,
    path: PathBuf,
}

impl fmt::Display for DecryptedPassword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(passphrase) = &self.passphrase {
            write!(f, "{}\n", passphrase)?;
        }
        for line in &self.lines {
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
        let mut lines = Vec::new();

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
                    lines.push(PasswordLine::Entry(key, value));
                }
                Rule::comment => {
                    lines.push(PasswordLine::Comment(record.as_str().to_owned()));
                }
                _ => unreachable!(),
            }
        }

        Ok(Self {
            passphrase,
            lines,
            path: path.to_owned(),
        })
    }

    pub(crate) fn create_and_write(
        passphrase: Option<String>,
        lines: Vec<PasswordLine>,
        path: &Path,
    ) -> Result<Self, StoreError> {
        let me = Self {
            passphrase,
            lines,
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
    pub fn generator(&mut self) -> PassphraseGenerator<()> {
        PassphraseGenerator::new(move |passphrase| self.set_passphrase(passphrase))
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn analyze_passphrase(&self) -> Option<AnalyzedPassphrase> {
        let passphrase = self.passphrase()?;

        Some(AnalyzedPassphrase::new(passphrase))
    }

    pub fn batch_edit(&mut self) -> DecryptedPasswordBatchEdit {
        DecryptedPasswordBatchEdit::new(
            self.passphrase.clone(),
            self.lines.clone(),
            move |passphrase, lines| {
                let old_passphrase = std::mem::replace(&mut self.passphrase, passphrase);
                let old_lines = std::mem::replace(&mut self.lines, lines);

                match self.save() {
                    Ok(()) => Ok(()),
                    Err(err) => {
                        self.passphrase = old_passphrase;
                        self.lines = old_lines;
                        Err(err)
                    }
                }
            },
        )
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

    pub fn lines(&self) -> impl Iterator<Item = (Position, &PasswordLine)> {
        self.lines
            .iter()
            .enumerate()
    }

    pub fn insert_line(&mut self, position: Position, line: PasswordLine) -> Result<(), StoreError> {
        let old_lines = self.lines.clone();
        self.lines.insert(position, line);

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn replace_line(&mut self, position: Position, line: PasswordLine) -> Result<(), StoreError> {
        let old_lines = self.lines.clone();
        if let Some(old_line) = self.lines.get_mut(position) {
            *old_line = line;
        } else {
            self.lines.push(line);
        }

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn remove_line(&mut self, position: Position) -> Result<(), StoreError> {
        let old_lines = self.lines.clone();
        self.lines.remove(position);

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn append_line(&mut self, line: PasswordLine) -> Result<(), StoreError> {
        let old_lines = self.lines.clone();
        self.lines.push(line);

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn prepend_line(&mut self, line: PasswordLine) -> Result<(), StoreError> {
        let old_lines = self.lines.clone();
        self.lines.insert(0, line);

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn set_lines(&mut self, lines: Vec<PasswordLine>) -> Result<(), StoreError> {
        let old_lines = std::mem::replace(&mut self.lines, lines);

        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn comments(&self) -> impl Iterator<Item = (Position, &str)> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(position, line)| {
                match line {
                    PasswordLine::Comment(comment) => Some((position, comment.as_str())),
                    PasswordLine::Entry(..) => None,
                }
            })
    }

    pub fn insert_comment<C: Into<String>>(
        &mut self,
        position: Position,
        comment: C,
    ) -> Result<(), StoreError> {
        self.insert_line(position, PasswordLine::Comment(comment.into()))
    }

    pub fn replace_comment<C: Into<String>>(&mut self, position: Position, comment: C) -> Result<(), StoreError> {
        if let Some(PasswordLine::Entry(..)) = self.lines.get(position) {
            return Err(StoreError::PasswordLineNotAComment(position));
        }
        self.replace_line(position, PasswordLine::Comment(comment.into()))
    }

    pub fn remove_comment(&mut self, position: Position) -> Result<(), StoreError> {
        if let Some(PasswordLine::Entry(..)) = self.lines.get(position) {
            return Err(StoreError::PasswordLineNotAComment(position));
        }
        self.remove_line(position)
    }

    pub fn prepend_comment<C: Into<String>>(&mut self, comment: C) -> Result<(), StoreError> {
        self.prepend_line(PasswordLine::Comment(comment.into()))
    }

    pub fn append_comment<C: Into<String>>(&mut self, comment: C) -> Result<(), StoreError> {
        self.append_line(PasswordLine::Comment(comment.into()))
    }

    pub fn all_entries(&self) -> impl Iterator<Item = (Position, (&str, &str))> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(position, line)| {
                match line {
                    PasswordLine::Entry(key, value) => Some((position, (key.as_str(), value.as_str()))),
                    PasswordLine::Comment(..) => None,
                }
            })
    }

    pub fn entry(&self, key: &str) -> Option<(Position, &str)> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(position, line)| {
                match line {
                    PasswordLine::Entry(k, v) if k == key => Some((position, v.as_str())),
                    _ => None,
                }
            })
            .next()
    }

    pub fn insert_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        position: Position,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        self.insert_line(position, PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn replace_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        position: Position,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        if let Some(PasswordLine::Comment(..)) = self.lines.get(position) {
            return Err(StoreError::PasswordLineNotAnEntry(position));
        }
        self.replace_line(position, PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn remove_entry(&mut self, position: Position) -> Result<(), StoreError> {
        if let Some(PasswordLine::Comment(..)) = self.lines.get(position) {
            return Err(StoreError::PasswordLineNotAnEntry(position));
        }
        self.remove_line(position)
    }

    pub fn prepend_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        self.prepend_line(PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn append_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        self.append_line(PasswordLine::Entry(key.into(), value.into()))
    }
}

pub struct DecryptedPasswordBatchEdit<'a> {
    passphrase: Option<String>,
    lines: Vec<PasswordLine>,
    batch_handler: Box<
        dyn 'a
            + FnOnce(
                Option<String>,
                Vec<PasswordLine>,
            ) -> Result<(), StoreError>,
    >,
}

impl<'a> DecryptedPasswordBatchEdit<'a> {
    fn new<
        F: 'a
            + FnOnce(
                Option<String>,
                Vec<PasswordLine>,
            ) -> Result<(), StoreError>,
    >(
        passphrase: Option<String>,
        lines: Vec<PasswordLine>,
        batch_handler: F,
    ) -> Self {
        Self {
            passphrase,
            lines,
            batch_handler: Box::new(batch_handler),
        }
    }

    pub fn passphrase<P: Into<String>>(mut self, passphrase: P) -> Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    pub fn lines(mut self, lines: Vec<PasswordLine>) -> Self {
        self.lines = lines;
        self
    }

    pub fn insert_line(mut self, position: Position, line: PasswordLine) -> Self {
        self.lines.insert(position, line);
        self
    }

    pub fn replace_line(mut self, position: Position, line: PasswordLine) -> Self {
        if let Some(old_line) = self.lines.get_mut(position) {
            *old_line = line;
        } else {
            self.lines.push(line);
        }

        self
    }

    pub fn remove_line(mut self, position: Position) -> Self {
        self.lines.remove(position);
        self
    }

    pub fn append_line(mut self, line: PasswordLine) -> Self {
        self.lines.push(line);
        self
    }

    pub fn prepend_line(mut self, line: PasswordLine) -> Self {
        self.lines.insert(0, line);
        self
    }

    pub fn insert_comment<C: Into<String>>(self, position: Position, comment: C) -> Self {
        self.insert_line(position, PasswordLine::Comment(comment.into()))
    }

    pub fn replace_comment<C: Into<String>>(self, position: Position, comment: C) -> Self {
        if let Some(PasswordLine::Entry(..)) = self.lines.get(position) {
            panic!(format!("Line at position {0} is not a comment!", position));
        }
        self.replace_line(position, PasswordLine::Comment(comment.into()))
    }

    pub fn remove_comment(self, position: Position) -> Self {
        if let Some(PasswordLine::Entry(..)) = self.lines.get(position) {
            panic!(format!("Line at position {0} is not a comment!", position));
        }
        self.remove_line(position)
    }

    pub fn append_comment<C: Into<String>>(self, comment: C) -> Self {
        self.append_line(PasswordLine::Comment(comment.into()))
    }

    pub fn prepend_comment<C: Into<String>>(self, comment: C) -> Self {
        self.prepend_line(PasswordLine::Comment(comment.into()))
    }

    pub fn insert_entry<K: Into<String>, V: Into<String>>(self, position: Position, key: K, value: V) -> Self {
        self.insert_line(position, PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn replace_entry<K: Into<String>, V: Into<String>>(self, position: Position, key: K, value: V) -> Self {
        if let Some(PasswordLine::Comment(..)) = self.lines.get(position) {
            panic!(format!("Line at position {0} is not an entry!", position));
        }
        self.replace_line(position, PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn remove_entry(self, position: Position) -> Self {
        if let Some(PasswordLine::Comment(..)) = self.lines.get(position) {
            panic!(format!("Line at position {0} is not an entry!", position));
        }
        self.remove_line(position)
    }

    pub fn append_entry<K: Into<String>, V: Into<String>>(self, key: K, value: V) -> Self {
        self.append_line(PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn prepend_entry<K: Into<String>, V: Into<String>>(self, key: K, value: V) -> Self {
        self.prepend_line(PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn edit(self) -> Result<(), StoreError> {
        (self.batch_handler)(self.passphrase, self.lines)
    }
}
