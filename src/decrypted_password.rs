use crate::{IntoStoreError, Store, StoreError};
use gpgme::{Context, Key, Protocol};
use std::{
    fmt,
    fs::{File, OpenOptions},
    io::Read,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

#[cfg(feature = "passphrase-utils")]
use crate::passphrase_utils::{AnalyzedPassphrase, PassphraseGenerator};

pub(crate) fn pw_name(path: &Path, store: &Store) -> String {
    path.strip_prefix(store.location())
        .expect("Password not stored inside this password store!")
        .with_extension("")
        .display()
        .to_string()
}

pub(crate) fn search_gpg_ids(mut path: &Path, ctx: &mut Context) -> Result<Vec<Key>, StoreError> {
    let original_path = path.to_owned();
    loop {
        if path.is_dir() && path.join(".gpg-id").is_file() {
            let mut file = OpenOptions::new()
                .read(true)
                .open(path.join(".gpg-id"))
                .expect("Failed to read .gpg-id");
            let mut content = String::new();
            file.read_to_string(&mut content).expect("not valid utf-8");

            return content
                .lines()
                .map(|line| {
                    ctx.get_key(line).map_err(|err| StoreError::Gpg("GPG-ID of .gpg-id file not found".to_owned(), err))
                })
                .collect();
        }

        if let Some(parent) = path.parent() {
            path = parent
        } else {
            return Err(StoreError::NoGpgId(original_path.display().to_string()));
        }
    }
}

pub(crate) fn save_password_to_file(
    store: &mut Store,
    path: &Path,
    password: impl fmt::Display,
    summary: Option<String>,
    changes: Vec<String>,
) -> Result<(), StoreError> {
    let mut f = NamedTempFile::new_in(path.parent().unwrap())
        .with_store_error(path.display().to_string())?;

    let mut ctx =
        Context::from_protocol(Protocol::OpenPgp).with_store_error(path.display().to_string())?;
    let content = format!("{}", password);
    let mut encrypted = Vec::new();
    let gpg_ids = search_gpg_ids(path, &mut ctx)?;
    let result = ctx
        .encrypt(gpg_ids.iter(), content, &mut encrypted)
        .with_store_error(path.display().to_string())?;
    if result.invalid_recipients().count() > 0 {
        return Err(StoreError::Gpg(
            "Could not encrypt for all gpg-id's".to_owned(),
            gpgme::Error::BAD_PUBKEY,
        ));
    }
    if encrypted.len() <= 0 {
        return Err(StoreError::Gpg(
            format!("Could not encrypt {}", path.display().to_string()),
            gpgme::Error::NOT_ENCRYPTED,
        ));
    }

    f.write_all(&encrypted)
        .with_store_error(path.display().to_string())?;
    f.flush().with_store_error(path.display().to_string())?;

    f.persist(path)
        .with_store_error(path.display().to_string())?;

    let pw_name = pw_name(path, store);
    if let Some(git) = store.git() {
        // this store uses git
        git.add(&[path]).with_store_error("add")?;

        let mut message = if let Some(message) = summary {
            message
        } else {
            format!("Edit password for '{}' using libpass.", pw_name,)
        };

        if !changes.is_empty() {
            // FIXME: changes may leak information on the passwords to unencrypted git log
            //        => consider encrypting `changes`
            message = format!("{}\n\n{}\n", message, changes.join("\n"));
        }

        git.commit(message).with_store_error("commit")?;
    }

    Ok(())
}

pub type Position = usize;

#[derive(Debug)]
pub struct DecryptedPassword {
    lines: Vec<String>,
    path: PathBuf,
    changes: Vec<String>,
}

impl fmt::Display for DecryptedPassword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            write!(f, "{}\n", line)?;
        }
        Ok(())
    }
}

impl DecryptedPassword {
    pub(crate) fn from_path(path: &Path) -> Result<Self, StoreError> {
        let mut pw = File::open(path).with_store_error(path.display().to_string())?;
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)
            .with_store_error("creating OpenPGP context")?;
        let mut content = Vec::new();
        // TODO: Add passphrase provider
        ctx.decrypt(&mut pw, &mut content)
            .with_store_error(path.display().to_string())?;

        let lines = String::from_utf8_lossy(&content)
            .lines()
            .map(|line| line.to_owned())
            .collect::<Vec<String>>();

        Ok(Self {
            lines,
            path: path.to_owned(),
            changes: Vec::new(),
        })
    }

    pub(crate) fn create_and_write(
        lines: Vec<String>,
        path: &Path,
        changes: Vec<String>,
        store: &mut Store,
    ) -> Result<Self, StoreError> {
        let mut me = Self {
            lines,
            path: path.to_owned(),
            changes,
        };
        me.save(
            Some(format!(
                "Add password for '{}' using libpass.",
                pw_name(path, store),
            )),
            store,
        )?;
        Ok(me)
    }

    fn save(&mut self, summary: Option<String>, store: &mut Store) -> Result<(), StoreError> {
        let changes = std::mem::replace(&mut self.changes, Vec::new());
        save_password_to_file(store, &self.path, &self, summary, changes)?;
        Ok(())
    }

    #[cfg(feature = "parsed-passwords")]
    pub fn parsed(self) -> Result<crate::parsed::DecryptedPassword, StoreError> {
        crate::parsed::DecryptedPassword::from_lines(self.lines, self.changes, self.path)
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn generator<'a>(&'a mut self, store: &'a mut Store) -> PassphraseGenerator<'a, ()> {
        PassphraseGenerator::new(move |passphrase| {
            self.changes
                .push("Set generated passphrase for password".into());
            self.replace_line(store, 0, passphrase).map(|_| ())
        })
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn analyze_passphrase(&self) -> Option<AnalyzedPassphrase> {
        let passphrase = self.passphrase()?;

        Some(AnalyzedPassphrase::new(passphrase))
    }

    pub fn batch_edit<'a>(&'a mut self, store: &'a mut Store) -> DecryptedPasswordBatchEdit<'a> {
        DecryptedPasswordBatchEdit::new(
            self.lines.clone(),
            self.changes.clone(),
            move |lines, changes| {
                self.changes = changes;
                self.set_lines(store, lines)
            },
        )
    }

    pub fn passphrase(&self) -> Option<&str> {
        self.lines.first().map(|p| p.as_str())
    }

    pub fn set_passphrase<P: Into<String>>(
        &mut self,
        store: &mut Store,
        passphrase: P,
    ) -> Result<(), StoreError> {
        self.changes
            .push("Set given passphrase for password".into());
        self.replace_line(store, 0, passphrase).map(|_| ())
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(|line| line.as_str())
    }

    pub fn set_lines<L: Into<Vec<String>>>(
        &mut self,
        store: &mut Store,
        lines: L,
    ) -> Result<(), StoreError> {
        let old_lines = std::mem::replace(&mut self.lines, lines.into());
        match self.save(None, store) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn replace_line<L: Into<String>>(
        &mut self,
        store: &mut Store,
        position: Position,
        line: L,
    ) -> Result<Option<String>, StoreError> {
        let old_lines = self.lines.clone();
        let removed_line: Option<String>;
        if let Some(old_line) = self.lines.get_mut(position) {
            removed_line = Some(std::mem::replace(old_line, line.into()));
        } else {
            removed_line = None;
            self.lines.push(line.into());
        }

        match self.save(None, store) {
            Ok(()) => Ok(removed_line),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn insert_line<L: Into<String>>(
        &mut self,
        store: &mut Store,
        position: Position,
        line: L,
    ) -> Result<(), StoreError> {
        let old_lines = self.lines.clone();
        self.lines.insert(position, line.into());
        match self.save(None, store) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn remove_line(&mut self, store: &mut Store, position: Position) -> Result<String, StoreError> {
        let old_lines = self.lines.clone();
        let removed_line = self.lines.remove(position);
        match self.save(None, store) {
            Ok(()) => Ok(removed_line),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn prepend_line<L: Into<String>>(
        &mut self,
        store: &mut Store,
        line: L,
    ) -> Result<(), StoreError> {
        self.insert_line(store, 1, line)
    }

    pub fn append_line<L: Into<String>>(
        &mut self,
        store: &mut Store,
        line: L,
    ) -> Result<(), StoreError> {
        self.insert_line(store, self.lines.len(), line)
    }
}

pub struct DecryptedPasswordBatchEdit<'a> {
    lines: Vec<String>,
    changes: Vec<String>,
    batch_handler: Box<dyn 'a + FnOnce(Vec<String>, Vec<String>) -> Result<(), StoreError>>,
}

impl<'a> DecryptedPasswordBatchEdit<'a> {
    fn new<F: 'a + FnOnce(Vec<String>, Vec<String>) -> Result<(), StoreError>>(
        lines: Vec<String>,
        changes: Vec<String>,
        batch_handler: F,
    ) -> Self {
        Self {
            lines,
            changes,
            batch_handler: Box::new(batch_handler),
        }
    }

    pub fn passphrase<P: Into<String>>(mut self, passphrase: P) -> Self {
        self.changes
            .push("Set given passphrase for password".into());
        if let Some(old_passphrase) = self.lines.get_mut(0) {
            *old_passphrase = passphrase.into();
        } else {
            self.lines.push(passphrase.into());
        }

        self
    }

    pub fn lines<L: Into<Vec<String>>>(mut self, lines: L) -> Self {
        self.lines = lines.into();
        self
    }

    pub fn insert_line<L: Into<String>>(mut self, position: Position, line: L) -> Self {
        self.lines.insert(position, line.into());
        self
    }

    pub fn replace_line<L: Into<String>>(mut self, position: Position, line: L) -> Self {
        if let Some(old_line) = self.lines.get_mut(position) {
            *old_line = line.into();
        } else {
            self.lines.push(line.into());
        }

        self
    }

    pub fn remove_line<L: Into<String>>(mut self, position: Position) -> Self {
        self.lines.remove(position);
        self
    }

    pub fn append_line<L: Into<String>>(mut self, line: L) -> Self {
        self.lines.push(line.into());
        self
    }

    pub fn prepend_line<L: Into<String>>(mut self, line: L) -> Self {
        self.lines.insert(0, line.into());
        self
    }

    pub fn edit(self) -> Result<(), StoreError> {
        (self.batch_handler)(self.lines, self.changes)
    }
}
