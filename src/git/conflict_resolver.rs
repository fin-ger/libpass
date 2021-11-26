use std::path::PathBuf;
use std::{ffi::OsStr, path::Path};

// this is a unix password store, so it is reasonable to assume unix here
use std::os::unix::ffi::OsStrExt;

use super::{GitResult, conflicted_binary::ConflictedBinary, conflicted_gpg_id::ConflictedGpgId, conflicted_password::ConflictedPassword, conflicted_plain_text::ConflictedPlainText};

#[derive(Debug)]
pub(super) struct ConflictEntry {
    pub(super) index_entry: git2::IndexEntry,
    pub(super) content: Vec<u8>,
    pub(super) path: PathBuf,
}

impl Clone for ConflictEntry {
    fn clone(&self) -> Self {
        let index_entry = git2::IndexEntry {
            ctime: self.index_entry.ctime.clone(),
            mtime: self.index_entry.mtime.clone(),
            dev: self.index_entry.dev,
            ino: self.index_entry.ino,
            mode: self.index_entry.mode,
            uid: self.index_entry.uid,
            gid: self.index_entry.gid,
            file_size: self.index_entry.file_size,
            id: self.index_entry.id.clone(),
            flags: self.index_entry.flags,
            flags_extended: self.index_entry.flags_extended,
            path: self.index_entry.path.clone(),
        };
        Self { index_entry, content: self.content.clone(), path: self.path.clone() }
    }
}

pub struct ConflictResolver<'a> {
    conflicted_passwords: Vec<ConflictedPassword>,
    conflicted_gpg_ids: Vec<ConflictedGpgId>,
    conflicted_plain_texts: Vec<ConflictedPlainText>,
    conflicted_binaries: Vec<ConflictedBinary>,
    finish_cb: Box<dyn FnOnce(&'a git2::Repository, Option<git2::Index>) -> GitResult<()> + 'a>,
    pub(super) maybe_index: Option<git2::Index>,
    pub(super) repository: &'a git2::Repository,
}

impl<'a> std::fmt::Debug for ConflictResolver<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConflictResolver")
            .field("conflicted_passwords", &self.conflicted_passwords)
            .field("conflicted_gpg_ids", &self.conflicted_gpg_ids)
            .field("conflicted_plain_texts", &self.conflicted_plain_texts)
            .field("conflicted_binaries", &self.conflicted_binaries)
            .field("finish_cb", &String::from("Box<dyn FnOnce(&'a git2::Repository, Option<git2::Index>) -> GitResult<()> + 'a>"))
            .field("maybe_index", &self.maybe_index.as_ref().map(|_| "Index"))
            .field("repository", &String::from("GitRepository"))
            .finish()
    }
}

impl<'a> ConflictResolver<'a> {
    pub(crate) fn new_without_conflicts(repo: &'a git2::Repository, finish_cb: impl FnOnce(&'a git2::Repository, Option<git2::Index>) -> GitResult<()> + 'a) -> Self {
        Self {
            conflicted_passwords: Vec::new(),
            conflicted_gpg_ids: Vec::new(),
            conflicted_plain_texts: Vec::new(),
            conflicted_binaries: Vec::new(),
            finish_cb: Box::new(finish_cb),
            maybe_index: None,
            repository: repo,
        }
    }

    pub(crate) fn from_index(index: git2::Index, repo: &'a git2::Repository, finish_cb: impl FnOnce(&'a git2::Repository, Option<git2::Index>) -> GitResult<()> + 'a) -> GitResult<Self> {
        let mut conflicted_passwords = Vec::new();
        let mut conflicted_gpg_ids = Vec::new();
        let mut conflicted_plain_texts = Vec::new();
        let mut conflicted_binaries = Vec::new();

        'conflicts:
        for conflict in index.conflicts()? {
            let conflict = conflict?;
            let ancestor = conflict.ancestor.unwrap();
            let our = conflict.our.unwrap();
            let their = conflict.their.unwrap();
            let ancestor_content = repo.find_blob(ancestor.id).expect("Blob of conflict not in repo").content().to_vec();
            let ancestor_path = Path::new(OsStr::from_bytes(&ancestor.path)).to_owned();
            let our_content = repo.find_blob(our.id).expect("Blob of conflict not in repo").content().to_vec();
            let our_path = Path::new(OsStr::from_bytes(&our.path)).to_owned();
            let their_content = repo.find_blob(their.id).expect("Blob of conflict not in repo").content().to_vec();
            let their_path = Path::new(OsStr::from_bytes(&their.path)).to_owned();

            let ancestor_entry = ConflictEntry {
                index_entry: ancestor,
                content: ancestor_content,
                path: ancestor_path,
            };
            let our_entry = ConflictEntry {
                index_entry: our,
                content: our_content,
                path: our_path.clone(),
            };
            let their_entry = ConflictEntry {
                index_entry: their,
                content: their_content,
                path: their_path.clone(),
            };

            if our_path.ends_with(".gpg") || their_path.ends_with(".gpg") {
                let conflicted_password_res = ConflictedPassword::new(ancestor_entry.clone(), our_entry.clone(), their_entry.clone());
                if conflicted_password_res.is_some() {
                    conflicted_passwords.push(conflicted_password_res.unwrap());
                    continue 'conflicts;
                }
            }

            if our_path.ends_with(".gpg-id") || their_path.ends_with(".gpg-id") {
                let conflicted_gpg_id = ConflictedGpgId::new(ancestor_entry.clone(), our_entry.clone(), their_entry.clone());
                if conflicted_gpg_id.is_some() {
                    conflicted_gpg_ids.push(conflicted_gpg_id.unwrap());
                    continue 'conflicts;
                }
            }

            let conflicted_plain_text = ConflictedPlainText::new(ancestor_entry.clone(), our_entry.clone(), their_entry.clone());
            if conflicted_plain_text.is_some() {
                conflicted_plain_texts.push(conflicted_plain_text.unwrap());
                continue 'conflicts;
            }

            conflicted_binaries.push(ConflictedBinary::new(ancestor_entry, our_entry, their_entry));
        }

        Ok(Self {
            conflicted_passwords,
            conflicted_gpg_ids,
            conflicted_plain_texts,
            conflicted_binaries,
            finish_cb: Box::new(finish_cb),
            maybe_index: Some(index),
            repository: repo,
        })
    }

    pub fn conflicted_passwords(&self) -> &Vec<ConflictedPassword> {
        &self.conflicted_passwords
    }

    pub fn conflicted_gpg_ids(&self) -> &Vec<ConflictedGpgId> {
        &self.conflicted_gpg_ids
    }

    pub fn conflicted_plain_texts(&self) -> &Vec<ConflictedPlainText> {
        &self.conflicted_plain_texts
    }

    pub fn conflicted_binaries(&self) -> &Vec<ConflictedBinary> {
        &self.conflicted_binaries
    }

    pub fn finish(self) -> GitResult<()> {
        (self.finish_cb)(self.repository, self.maybe_index)
    }
}
