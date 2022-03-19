use std::{fmt, fs::OpenOptions, io::Read, path::{Path, PathBuf}};

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
    let root = repo.path().parent().unwrap();
    loop {
        if let Some(gpg_id_index_entry) = idx.get_path(&path.join(".gpg-id"), -1) {
            let blob = repo.find_blob(gpg_id_index_entry.id).expect("gpg-id blob not in repository");
            let content = String::from_utf8(blob.content().to_vec()).expect("gpg-id not valid utf-8");

            return content
                .lines()
                .map(|line| {
                    ctx.get_key(line).map_err(|err| StoreError::Gpg("GPG-ID of .gpg-id file not found".to_owned(), err))
                }).collect();
        } else if root.join(path).is_dir() && root.join(path).join(".gpg-id").is_file() {
            let mut file = OpenOptions::new()
                .read(true)
                .open(root.join(path).join(".gpg-id"))
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

#[derive(Debug, Clone)]
pub struct ConflictedPassword {
    ancestor: Option<ConflictEntry>,
    our: Option<ConflictEntry>,
    their: Option<ConflictEntry>,
    ancestor_password: Option<ConflictedDecryptedPassword>,
    our_password: Option<ConflictedDecryptedPassword>,
    their_password: Option<ConflictedDecryptedPassword>,
    is_resolved: bool,
}

impl ConflictedPassword {
    pub(super) fn new(ancestor: Option<ConflictEntry>, our: Option<ConflictEntry>, their: Option<ConflictEntry>) -> Option<Self> {
        let ancestor_password = if let Some(ancestor) = &ancestor {
            Some(ConflictedDecryptedPassword::from_buffer(&ancestor.content, &ancestor.path).ok()?)
        } else {
            None
        };
        let our_password = if let Some(our) = &our {
            Some(ConflictedDecryptedPassword::from_buffer(&our.content, &our.path).ok()?)
        } else {
            None
        };
        let their_password = if let Some(their) = &their {
            Some(ConflictedDecryptedPassword::from_buffer(&their.content, &their.path).ok()?)
        } else {
            None
        };

        Some(Self {
            ancestor_password,
            our_password,
            their_password,
            ancestor,
            our,
            their,
            is_resolved: false,
        })
    }

    pub fn ancestor_path(&self) -> Option<&Path> {
        if let Some(ancestor) = &self.ancestor {
            Some(&ancestor.path)
        } else {
            None
        }
    }

    pub fn our_path(&self) -> Option<&Path> {
        if let Some(our) = &self.our {
            Some(&our.path)
        } else {
            None
        }
    }

    pub fn their_path(&self) -> Option<&Path> {
        if let Some(their) = &self.their {
            Some(&their.path)
        } else {
            None
        }
    }

    pub fn ancestor_password(&self) -> Option<ConflictedDecryptedPassword> {
        // clone password, so it can be parsed
        self.ancestor_password.clone()
    }

    pub fn our_password(&self) -> Option<ConflictedDecryptedPassword> {
        // clone password, so it can be parsed
        self.our_password.clone()
    }

    pub fn their_password(&self) -> Option<ConflictedDecryptedPassword> {
        // clone password, so it can be parsed
        self.their_password.clone()
    }

    pub fn resolve(&mut self, conflict_resolver: &mut ConflictResolver, resolved_password: Option<ConflictedDecryptedPassword>) -> Result<(), StoreError> {
        if self.is_resolved {
            return Err(git2::Error::new(git2::ErrorCode::Invalid, git2::ErrorClass::Merge, "Merge conflict already resolved"))
                .with_store_error("Resolve merge conflict");
        }

        let mut entries = self.our.iter()
            .chain(self.ancestor.iter())
            .chain(self.their.iter());
        let mut conflict_entry = entries
            .next()
            .expect("No index entry available for conflict resolution. So it finally happened... Couldn't produce a test case triggering this behavior and libgit2 docs say nothing about it. Please report it in libpass's issue tracker!")
            .clone();

        if let Some(resolved_password) = resolved_password {
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
            let oid = conflict_resolver.repository.blob(&encrypted)
                .with_store_error("add resolved password blob to repository")?;
            conflict_entry.index_entry.file_size = encrypted.len() as u32;
            conflict_entry.index_entry.id = oid;

            let index = conflict_resolver.maybe_index.as_mut()
                .expect("Conflict resolver has no index set when trying to resolve conflict");
            index.add(&conflict_entry.index_entry)
                .with_store_error("add resolved merge conflict to index")?;
        } else {
            let index = conflict_resolver.maybe_index.as_mut()
                .expect("Conflict resolver has no index set when trying to resolve conflict");
            // stage is ANY (-1) to remove it from ancestor, ours, and theirs
            index.remove(&conflict_entry.path, -1)
                .with_store_error("remove resolved merge conflict from index")?;
        }

        let index = conflict_resolver.maybe_index.as_mut()
            .expect("Conflict resolver has no index set when trying to resolve conflict");
        index.conflict_remove(&conflict_entry.path)
            .with_store_error("remove conflict from repository")?;

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

        let lines = String::from_utf8_lossy(&decrypted)
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

    pub fn diff<'a>(&'a self, other: &'a ConflictedDecryptedPassword) -> Vec<PasswordChange<'a>> {
        let my_lines = self.lines().collect::<Vec<&str>>();
        let other_lines = other.lines().collect::<Vec<&str>>();

        similar::capture_diff_slices(similar::Algorithm::Myers, &my_lines, &other_lines)
            .drain(..).map(|diff_op| {
                match diff_op {
                    similar::DiffOp::Equal { old_index, new_index, len } => {
                        let lines = my_lines[old_index..old_index+len].into_iter()
                            .enumerate().map(|(idx, line)| PasswordLine {
                                content: line,
                                my_linum: old_index + idx,
                                other_linum: new_index + idx,
                            })
                            .collect::<Vec<PasswordLine>>();
                        PasswordChange::Equal(lines)
                    },
                    similar::DiffOp::Delete { old_index, old_len, new_index } => {
                        let lines = my_lines[old_index..old_index+old_len].into_iter()
                            .enumerate().map(|(idx, line)| PasswordLine {
                                content: line,
                                my_linum: old_index + idx,
                                other_linum: new_index,
                            })
                            .collect::<Vec<PasswordLine>>();
                        PasswordChange::Delete(lines)
                    },
                    similar::DiffOp::Insert { old_index, new_index, new_len } => {
                        let lines: Vec<PasswordLine> = other_lines[new_index..new_index+new_len].into_iter()
                            .enumerate().map(|(idx, line)| PasswordLine {
                                content: line,
                                my_linum: old_index,
                                other_linum: new_index + idx,
                            })
                            .collect();
                        PasswordChange::Insert(lines)
                    },
                    similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                        let my_lines = my_lines[old_index..old_index+old_len].into_iter()
                            .enumerate().map(|(idx, line)| PasswordLine {
                                content: line,
                                my_linum: old_index + idx,
                                other_linum: new_index,
                            })
                            .collect::<Vec<PasswordLine>>();
                        let other_lines = other_lines[new_index..new_index+new_len].into_iter()
                            .enumerate().map(|(idx, line)| PasswordLine {
                                content: line,
                                my_linum: old_index,
                                other_linum: new_index + idx,
                            })
                            .collect::<Vec<PasswordLine>>();
                        PasswordChange::Replace { my_lines, other_lines }
                    },
                }
            })
            .collect::<Vec<PasswordChange>>()
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

#[derive(Debug)]
pub struct PasswordLine<'a> {
    pub content: &'a str,
    pub my_linum: usize,
    pub other_linum: usize,
}

#[derive(Debug)]
pub enum PasswordChange<'a> {
    Equal(Vec<PasswordLine<'a>>),
    Delete(Vec<PasswordLine<'a>>),
    Insert(Vec<PasswordLine<'a>>),
    Replace {
        my_lines: Vec<PasswordLine<'a>>,
        other_lines: Vec<PasswordLine<'a>>,
    },
}
