use std::{ffi::OsStr, fmt, path::{Path, PathBuf}};
use std::os::unix::ffi::OsStrExt;

use crate::{ConflictResolver, IntoStoreError, Position, StoreError};

use super::conflict_resolver::ConflictEntry;

#[cfg(feature = "passphrase-utils")]
use crate::passphrase_utils::AnalyzedPassphrase;

pub(crate) fn search_gpg_ids_in_index(mut path: &Path, repo: &git2::Repository, maybe_index: &Option<git2::Index>, ctx: &mut gpgme::Context) -> Result<Vec<gpgme::Key>, StoreError> {
    let idx = if let Some(idx) = maybe_index {
        idx
    } else {
        return Err(git2::Error::new(git2::ErrorCode::NotFound, git2::ErrorClass::Index, "Index not available for gpg-id search"))
            .with_store_error("Index path lookup");
    };

    let original_path = path.to_owned();
    loop {
        if let Some(gpg_id_index_entry) = idx.get_path(&path.join(".gpg-id"), -1) {
            let blob = repo.find_blob(gpg_id_index_entry.id).expect("gpg-id blob not in repository");
            let content = String::from_utf8(blob.content().to_vec()).expect("gpg-id not valid utf-8");

            return Ok(content
                      .lines()
                      .map(|line| ctx.get_key(line).expect("Key not found"))
                      .collect());
        }

        if let Some(parent) = path.parent() {
            path = parent
        } else {
            return Err(StoreError::NoGpgId(original_path.display().to_string()));
        }
    }
}

#[derive(Debug)]
pub struct ConflictedPassword {
    ancestor: ConflictEntry,
    our: ConflictEntry,
    their: ConflictEntry,
    ancestor_password: ConflictedDecryptedPassword,
    our_password: ConflictedDecryptedPassword,
    their_password: ConflictedDecryptedPassword,
    is_resolved: bool,
}

impl ConflictedPassword {
    pub(super) fn new(ancestor: ConflictEntry, our: ConflictEntry, their: ConflictEntry) -> Option<Self> {
        Some(Self {
            ancestor_password: ConflictedDecryptedPassword::from_buffer(&ancestor.content, &ancestor.path).ok()?,
            our_password: ConflictedDecryptedPassword::from_buffer(&our.content, &our.path).ok()?,
            their_password: ConflictedDecryptedPassword::from_buffer(&their.content, &their.path).ok()?,
            ancestor,
            our,
            their,
            is_resolved: false,
        })
    }

    pub fn ancestor_path(&self) -> &Path {
        &self.ancestor.path
    }

    pub fn our_path(&self) -> &Path {
        &self.our.path
    }

    pub fn their_path(&self) -> &Path {
        &self.their.path
    }

    pub fn ancestor_password(&self) -> ConflictedDecryptedPassword {
        // clone password, so it can be parsed
        self.ancestor_password.clone()
    }

    pub fn our_password(&self) -> ConflictedDecryptedPassword {
        // clone password, so it can be parsed
        self.our_password.clone()
    }

    pub fn their_password(&self) -> ConflictedDecryptedPassword {
        // clone password, so it can be parsed
        self.their_password.clone()
    }

    pub fn our_changes(&self) -> PasswordChanges {
        todo!();
    }

    pub fn their_changes(&self) -> PasswordChanges {
        todo!();
    }

    pub fn resolve(&mut self, conflict_resolver: &mut ConflictResolver, resolved_password: ConflictedDecryptedPassword) -> Result<(), StoreError> {
        if self.is_resolved {
            return Err(git2::Error::new(git2::ErrorCode::Invalid, git2::ErrorClass::Merge, "Merge conflict already resolved"))
                .with_store_error("Resolve merge conflict");
        }

        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
            .with_store_error(resolved_password.path.display().to_string())?;
        let content = format!("{}", resolved_password);
        let mut encrypted = Vec::new();
        let gpg_ids = search_gpg_ids_in_index(&resolved_password.path, &conflict_resolver.repository, &conflict_resolver.maybe_index, &mut ctx)?;
        let result = ctx
            .encrypt(gpg_ids.iter(), content, &mut encrypted)
            .with_store_error(resolved_password.path.display().to_string())?;
        if result.invalid_recipients().count() > 0 {
            return Err(StoreError::Gpg(
                "Could not encrypt for all gpg-id's".to_owned(),
                gpgme::Error::BAD_PUBKEY,
            ));
        }
        if encrypted.len() <= 0 {
            return Err(StoreError::Gpg(
                format!("Could not encrypt {}", resolved_password.path.display().to_string()),
                gpgme::Error::NOT_ENCRYPTED,
            ));
        }

        let index = conflict_resolver.maybe_index.as_mut()
            .expect("Conflict resolver has no index set when trying to resolve conflict");
        let entries = [&self.ancestor.index_entry, &self.our.index_entry, &self.their.index_entry];
        let index_entry = entries.iter()
            .find(|ie| Path::new(OsStr::from_bytes(&ie.path)) == &resolved_password.path)
            .expect("No index entry matches path of resolved password");
        index.add_frombuffer(index_entry, &encrypted)
            .with_store_error("add resolved merge conflict to index")?;
        self.is_resolved = true;

        Ok(())
    }

    pub fn is_resolved(&self) -> bool {
        self.is_resolved
    }
}

#[derive(Debug, Clone)]
pub struct ConflictedDecryptedPassword {
    lines: Vec<String>,
    recipients: Vec<String>,
    path: PathBuf,
}

impl fmt::Display for ConflictedDecryptedPassword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            write!(f, "{}\n", line)?;
        }
        Ok(())
    }
}

impl ConflictedDecryptedPassword {
    fn from_buffer(content: &[u8], path: &Path) -> gpgme::Result<Self> {
        let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)?;
        let mut decrypted = Vec::new();
        // TODO: Add passphrase provider
        let recipients = ctx.decrypt(content, &mut decrypted)?
            .recipients()
            .map(|recp| recp.key_id().expect("Key id not valid utf-8").to_owned())
            .collect();

        let lines = String::from_utf8_lossy(&content)
            .lines()
            .map(|line| line.to_owned())
            .collect::<Vec<String>>();

        Ok(Self {
            lines,
            recipients,
            path: path.to_owned(),
        })
    }

    #[cfg(feature = "parsed-passwords")]
    pub fn parsed(self) -> Result<crate::parsed::ConflictedDecryptedPassword, StoreError> {
        crate::parsed::ConflictedDecryptedPassword::from_lines(self.lines)
    }

    #[cfg(feature = "passphrase-utils")]
    pub fn analyze_passphrase(&self) -> Option<AnalyzedPassphrase> {
        let passphrase = self.passphrase()?;

        Some(AnalyzedPassphrase::new(passphrase))
    }

    pub fn passphrase(&self) -> Option<&str> {
        self.lines.first().map(|p| p.as_str())
    }

    pub fn set_passphrase<P: Into<String>>(
        &mut self,
        passphrase: P,
    ) {
        self.replace_line(0, passphrase);
    }

    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(|line| line.as_str())
    }

    pub fn set_lines<L: Into<Vec<String>>>(
        &mut self,
        lines: L,
    ) {
        let _old_lines = std::mem::replace(&mut self.lines, lines.into());
   }

    pub fn replace_line<L: Into<String>>(
        &mut self,
        position: Position,
        line: L,
    ) -> Option<String> {
        let removed_line: Option<String>;
        if let Some(old_line) = self.lines.get_mut(position) {
            removed_line = Some(std::mem::replace(old_line, line.into()));
        } else {
            removed_line = None;
            self.lines.push(line.into());
        }

        removed_line
    }

    pub fn insert_line<L: Into<String>>(
        &mut self,
        position: Position,
        line: L,
    ) {
        self.lines.insert(position, line.into());
    }

    pub fn remove_line(&mut self, position: Position) -> String {
        self.lines.remove(position)
    }

    pub fn prepend_line<L: Into<String>>(
        &mut self,
        line: L,
    ) {
        self.insert_line(1, line)
    }

    pub fn append_line<L: Into<String>>(
        &mut self,
        line: L,
    ) {
        self.insert_line(self.lines.len(), line)
    }
}

pub struct PasswordLine<'a> {
    content: &'a str,
    linum: u64,
}

pub enum PasswordChange<'a> {
    Equal(&'a [PasswordLine<'a>]),
    Delete(&'a [PasswordLine<'a>]),
    Insert(&'a [PasswordLine<'a>]),
}

pub struct PasswordChanges;
