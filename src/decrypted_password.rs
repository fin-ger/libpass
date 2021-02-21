use crate::{IntoStoreError, StoreError};
use gpgme::{Context, Protocol, Key};
use std::{
    fmt,
    fs::{File, OpenOptions},
    io::Read,
    io::Write,
    path::{Path, PathBuf},
};

#[cfg(feature = "passphrase-utils")]
use crate::passphrase_utils::{AnalyzedPassphrase, PassphraseGenerator};

pub(crate) fn search_gpg_ids(mut path: &Path, ctx: &mut Context) -> Vec<Key> {
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
                .map(|line| ctx.get_key(line).expect("Key not found"))
                .collect();
        }

        if let Some(parent) = path.parent() {
            path = parent
        } else {
            return Vec::new();
        }
    }
}

pub type Position = usize;

pub struct DecryptedPassword {
    lines: Vec<String>,
    path: PathBuf,
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
            .with_store_error(path.display().to_string())?;
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
        })
    }

    pub(crate) fn create_and_write(lines: Vec<String>, path: &Path) -> Result<Self, StoreError> {
        let me = Self {
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

    #[cfg(feature = "parsed-passwords")]
    pub fn parsed(self) -> Result<crate::parsed::DecryptedPassword, StoreError> {
        crate::parsed::DecryptedPassword::from_lines(self.lines, self.path)
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
        self.lines.first().map(|p| p.as_str())
    }

    pub fn set_passphrase<P: Into<String>>(&mut self, passphrase: P) -> Result<(), StoreError> {
        self.replace_line(0, passphrase).map(|_| ())
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(|line| line.as_str())
    }

    pub fn set_lines<L: Into<Vec<String>>>(&mut self, lines: L) -> Result<(), StoreError> {
        let old_lines = std::mem::replace(&mut self.lines, lines.into());
        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn replace_line<L: Into<String>>(
        &mut self,
        position: Position,
        line: L,
    ) -> Result<Option<String>, StoreError> {
        let old_lines = self.lines.clone();
        let removed_line: Option<String>;
        if let Some(old_line) = self.lines.get_mut(position) {
            removed_line = Some(std::mem::replace(old_line, line.into()));
        } else {
            removed_line = None;
            self.lines.insert(position, line.into());
        }

        match self.save() {
            Ok(()) => Ok(removed_line),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn insert_line<L: Into<String>>(
        &mut self,
        position: Position,
        line: L,
    ) -> Result<(), StoreError> {
        let old_lines = self.lines.clone();
        self.lines.insert(position, line.into());
        match self.save() {
            Ok(()) => Ok(()),
            Err(err) => {
                self.lines = old_lines;
                Err(err)
            }
        }
    }

    pub fn prepend_line<L: Into<String>>(&mut self, line: L) -> Result<(), StoreError> {
        self.insert_line(1, line)
    }

    pub fn append_line<L: Into<String>>(&mut self, line: L) -> Result<(), StoreError> {
        self.insert_line(self.lines.len(), line)
    }
}
