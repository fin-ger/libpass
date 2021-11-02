use pest::Parser;
use pest_derive::Parser;
use std::{
    fmt,
    path::{Path, PathBuf},
};

use crate::{Position, Store, StoreError, pw_name, save_password_to_file};

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
    changes: Vec<String>,
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
    pub(crate) fn from_lines(lines: Vec<String>, changes: Vec<String>, path: PathBuf) -> Result<Self, StoreError> {
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
            changes,
            path: path.to_owned(),
        })
    }

    pub(crate) fn create_and_write(
        passphrase: Option<String>,
        lines: Vec<PasswordLine>,
        path: &Path,
        changes: Vec<String>,
        store: &mut Store,
    ) -> Result<Self, StoreError> {
        let mut me = Self {
            passphrase,
            lines,
            changes,
            path: path.to_owned(),
        };
        me.save(Some(format!(
            "Add password for {} using libpass.",
            pw_name(path, store),
        )), store)?;
        Ok(me)
    }

    fn save(&mut self, summary: Option<String>, store: &mut Store) -> Result<(), StoreError> {
        let changes = std::mem::replace(&mut self.changes, Vec::new());
        save_password_to_file(store, &self.path, &self, summary, changes)?;
        Ok(())
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn generator<'a>(&'a mut self, store: &'a mut Store) -> PassphraseGenerator<'a, ()> {
        PassphraseGenerator::new(move |passphrase| {
            self.changes.push("Set generated passphrase for password".into());
            let old_passphrase = self.passphrase.replace(passphrase.into());
            match self.save(None, store) {
                Ok(()) => Ok(()),
                Err(err) => {
                    self.passphrase = old_passphrase;
                    Err(err)
                }
            }
        })
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn analyze_passphrase(&self) -> Option<AnalyzedPassphrase> {
        let passphrase = self.passphrase()?;

        Some(AnalyzedPassphrase::new(passphrase))
    }

    pub fn batch_edit<'a>(&'a mut self, store: &'a mut Store) -> DecryptedPasswordBatchEdit<'a> {
        DecryptedPasswordBatchEdit::new(
            self.passphrase.clone(),
            self.lines.clone(),
            self.changes.clone(),
            move |passphrase, lines, changes| {
                self.changes = changes;
                let old_passphrase = std::mem::replace(&mut self.passphrase, passphrase);
                let old_lines = std::mem::replace(&mut self.lines, lines);

                match self.save(None, store) {
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

    pub fn set_passphrase<P: Into<String>>(&mut self, store: &mut Store, passphrase: P) -> Result<(), StoreError> {
        self.changes.push("Set given passphrase for password".into());
        let old_passphrase = self.passphrase.replace(passphrase.into());
        match self.save(None, store) {
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

    pub fn insert_line(&mut self, store: &mut Store, position: Position, line: PasswordLine) -> Result<(), StoreError> {
        let message = match &line {
            PasswordLine::Comment(_) => "Add comment to password".into(),
            PasswordLine::Entry(key, _) => format!("Add {} entry to password", key),
        };
        self.changes.push(message);

        let old_lines = self.lines.clone();
        self.lines.insert(position, line);

        match self.save(None, store) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn replace_line(&mut self, store: &mut Store, position: Position, line: PasswordLine) -> Result<(), StoreError> {
        let message = match &line {
            PasswordLine::Comment(_) => "Replace comment in password".into(),
            PasswordLine::Entry(key, _) => format!("Replace {} entry in password", key),
        };
        self.changes.push(message);

        let old_lines = self.lines.clone();
        if let Some(old_line) = self.lines.get_mut(position) {
            *old_line = line;
        } else {
            self.lines.push(line);
        }

        match self.save(None, store) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn remove_line(&mut self, store: &mut Store, position: Position) -> Result<(), StoreError> {
        let old_lines = self.lines.clone();
        let old_line = self.lines.remove(position);

        let message = match &old_line {
            PasswordLine::Comment(_) => "Remove comment from password".into(),
            PasswordLine::Entry(key, _) => format!("Remove {} entry from password", key),
        };
        self.changes.push(message);

        match self.save(None, store) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn append_line(&mut self, store: &mut Store, line: PasswordLine) -> Result<(), StoreError> {
        let message = match &line {
            PasswordLine::Comment(_) => "Add comment to password".into(),
            PasswordLine::Entry(key, _) => format!("Add {} entry to password", key),
        };
        self.changes.push(message);

        let old_lines = self.lines.clone();
        self.lines.push(line);

        match self.save(None, store) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn prepend_line(&mut self, store: &mut Store, line: PasswordLine) -> Result<(), StoreError> {
        let message = match &line {
            PasswordLine::Comment(_) => "Add comment to password".into(),
            PasswordLine::Entry(key, _) => format!("Add {} entry to password", key),
        };
        self.changes.push(message);

        let old_lines = self.lines.clone();
        self.lines.insert(0, line);

        match self.save(None, store) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn set_lines(&mut self, store: &mut Store, lines: Vec<PasswordLine>) -> Result<(), StoreError> {
        let old_lines = std::mem::replace(&mut self.lines, lines);

        match self.save(None, store) {
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
        store: &mut Store,
        position: Position,
        comment: C,
    ) -> Result<(), StoreError> {
        self.insert_line(store, position, PasswordLine::Comment(comment.into()))
    }

    pub fn replace_comment<C: Into<String>>(&mut self, store: &mut Store, position: Position, comment: C) -> Result<(), StoreError> {
        if let Some(PasswordLine::Entry(..)) = self.lines.get(position) {
            return Err(StoreError::PasswordLineNotAComment(position));
        }
        self.replace_line(store, position, PasswordLine::Comment(comment.into()))
    }

    pub fn remove_comment(&mut self, store: &mut Store, position: Position) -> Result<(), StoreError> {
        if let Some(PasswordLine::Entry(..)) = self.lines.get(position) {
            return Err(StoreError::PasswordLineNotAComment(position));
        }
        self.remove_line(store, position)
    }

    pub fn prepend_comment<C: Into<String>>(&mut self, store: &mut Store, comment: C) -> Result<(), StoreError> {
        self.prepend_line(store, PasswordLine::Comment(comment.into()))
    }

    pub fn append_comment<C: Into<String>>(&mut self, store: &mut Store, comment: C) -> Result<(), StoreError> {
        self.append_line(store, PasswordLine::Comment(comment.into()))
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
        store: &mut Store,
        position: Position,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        self.insert_line(store, position, PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn replace_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        store: &mut Store,
        position: Position,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        if let Some(PasswordLine::Comment(..)) = self.lines.get(position) {
            return Err(StoreError::PasswordLineNotAnEntry(position));
        }
        self.replace_line(store, position, PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn remove_entry(&mut self, store: &mut Store, position: Position) -> Result<(), StoreError> {
        if let Some(PasswordLine::Comment(..)) = self.lines.get(position) {
            return Err(StoreError::PasswordLineNotAnEntry(position));
        }
        self.remove_line(store, position)
    }

    pub fn prepend_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        store: &mut Store,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        self.prepend_line(store, PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn append_entry<K: Into<String>, V: Into<String>>(
        &mut self,
        store: &mut Store,
        key: K,
        value: V,
    ) -> Result<(), StoreError> {
        self.append_line(store, PasswordLine::Entry(key.into(), value.into()))
    }
}

pub struct DecryptedPasswordBatchEdit<'a> {
    passphrase: Option<String>,
    lines: Vec<PasswordLine>,
    changes: Vec<String>,
    batch_handler: Box<
        dyn 'a
            + FnOnce(
                Option<String>,
                Vec<PasswordLine>,
                Vec<String>,
            ) -> Result<(), StoreError>,
    >,
}

impl<'a> DecryptedPasswordBatchEdit<'a> {
    fn new<
        F: 'a
            + FnOnce(
                Option<String>,
                Vec<PasswordLine>,
                Vec<String>,
            ) -> Result<(), StoreError>,
    >(
        passphrase: Option<String>,
        lines: Vec<PasswordLine>,
        changes: Vec<String>,
        batch_handler: F,
    ) -> Self {
        Self {
            passphrase,
            lines,
            changes,
            batch_handler: Box::new(batch_handler),
        }
    }

    pub fn passphrase<P: Into<String>>(mut self, passphrase: P) -> Self {
        self.changes.push("Set given passphrase for password".into());
        self.passphrase = Some(passphrase.into());
        self
    }

    pub fn lines(mut self, lines: Vec<PasswordLine>) -> Self {
        self.lines = lines;
        self
    }

    pub fn insert_line(mut self, position: Position, line: PasswordLine) -> Self {
        let message = match &line {
            PasswordLine::Comment(_) => "Add comment to password".into(),
            PasswordLine::Entry(key, _) => format!("Add {} entry to password", key),
        };
        self.changes.push(message);

        self.lines.insert(position, line);
        self
    }

    pub fn replace_line(mut self, position: Position, line: PasswordLine) -> Self {
        let message = match &line {
            PasswordLine::Comment(_) => "Replace comment in password".into(),
            PasswordLine::Entry(key, _) => format!("Replace {} entry in password", key),
        };
        self.changes.push(message);

        if let Some(old_line) = self.lines.get_mut(position) {
            *old_line = line;
        } else {
            self.lines.push(line);
        }

        self
    }

    pub fn remove_line(mut self, position: Position) -> Self {
        let old_line = self.lines.remove(position);

        let message = match &old_line {
            PasswordLine::Comment(_) => "Remove comment from password".into(),
            PasswordLine::Entry(key, _) => format!("Remove {} entry from password", key),
        };
        self.changes.push(message);

        self
    }

    pub fn append_line(mut self, line: PasswordLine) -> Self {
        let message = match &line {
            PasswordLine::Comment(_) => "Add comment to password".into(),
            PasswordLine::Entry(key, _) => format!("Add {} entry to password", key),
        };
        self.changes.push(message);

        self.lines.push(line);
        self
    }

    pub fn prepend_line(mut self, line: PasswordLine) -> Self {
        let message = match &line {
            PasswordLine::Comment(_) => "Add comment to password".into(),
            PasswordLine::Entry(key, _) => format!("Add {} entry to password", key),
        };
        self.changes.push(message);

        self.lines.insert(0, line);
        self
    }

    pub fn insert_comment<C: Into<String>>(self, position: Position, comment: C) -> Self {
        self.insert_line(position, PasswordLine::Comment(comment.into()))
    }

    pub fn replace_comment<C: Into<String>>(self, position: Position, comment: C) -> Self {
        if let Some(PasswordLine::Entry(..)) = self.lines.get(position) {
            panic!("Line at position {0} is not a comment!", position);
        }
        self.replace_line(position, PasswordLine::Comment(comment.into()))
    }

    pub fn remove_comment(self, position: Position) -> Self {
        if let Some(PasswordLine::Entry(..)) = self.lines.get(position) {
            panic!("Line at position {0} is not a comment!", position);
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
            panic!("Line at position {0} is not an entry!", position);
        }
        self.replace_line(position, PasswordLine::Entry(key.into(), value.into()))
    }

    pub fn remove_entry(self, position: Position) -> Self {
        if let Some(PasswordLine::Comment(..)) = self.lines.get(position) {
            panic!("Line at position {0} is not an entry!", position);
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
        (self.batch_handler)(self.passphrase, self.lines, self.changes)
    }
}
