use std::path::PathBuf;
use std::{ffi::OsStr, path::Path};

// this is a unix password store, so it is reasonable to assume unix here
use std::os::unix::ffi::OsStrExt;

use git2::IndexEntry;

use super::{GitResult, conflicted_binary::ConflictedBinary, conflicted_gpg_id::ConflictedGpgId, conflicted_password::ConflictedPassword, conflicted_plain_text::ConflictedPlainText};

#[derive(Debug)]
pub(super) struct ConflictEntry {
    pub(super) index_entry: git2::IndexEntry,
    pub(super) content: Vec<u8>,
    pub(super) path: PathBuf,
}

pub(crate) fn clone_index_entry(index_entry: &IndexEntry) -> IndexEntry {
    IndexEntry {
        ctime: index_entry.ctime.clone(),
        mtime: index_entry.mtime.clone(),
        dev: index_entry.dev,
        ino: index_entry.ino,
        mode: index_entry.mode,
        uid: index_entry.uid,
        gid: index_entry.gid,
        file_size: index_entry.file_size,
        id: index_entry.id.clone(),
        flags: index_entry.flags,
        flags_extended: index_entry.flags_extended,
        path: index_entry.path.clone(),
    }
}

impl ConflictEntry {
    fn from_index_entry(repo: &git2::Repository, maybe_index_entry: Option<IndexEntry>) -> Option<Self> {
        if let Some(index_entry) = maybe_index_entry {
            let content = repo.find_blob(index_entry.id).expect("Blob of conflict not in repo").content().to_vec();
            let path = Path::new(OsStr::from_bytes(&index_entry.path)).to_owned();

            Some(Self {
                index_entry,
                content,
                path: path.clone(),
            })
        } else {
            None
        }
    }
}

fn entry_has_extension(entry: &Option<ConflictEntry>, ext: &str) -> bool {
    let path = if let Some(entry) = entry {
        &entry.path
    } else {
        return false;
    };

    if let Some(path_ext) = path.extension().and_then(OsStr::to_str) {
        path_ext == ext
    } else if let Some(path) = path.file_name().and_then(OsStr::to_str) {
        path == format!(".{}", ext)
    } else {
        false
    }
}


impl Clone for ConflictEntry {
    fn clone(&self) -> Self {
        Self { index_entry: clone_index_entry(&self.index_entry), content: self.content.clone(), path: self.path.clone() }
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

            let ancestor_entry = ConflictEntry::from_index_entry(repo, conflict.ancestor);
            let our_entry = ConflictEntry::from_index_entry(repo, conflict.our);
            let their_entry = ConflictEntry::from_index_entry(repo, conflict.their);

            if entry_has_extension(&ancestor_entry, "gpg") ||
                entry_has_extension(&our_entry, "gpg") ||
                entry_has_extension(&their_entry, "gpg")
            {
                let conflicted_password_res = ConflictedPassword::new(ancestor_entry.clone(), our_entry.clone(), their_entry.clone());
                if conflicted_password_res.is_some() {
                    conflicted_passwords.push(conflicted_password_res.unwrap());
                    continue 'conflicts;
                }
            }

            if entry_has_extension(&ancestor_entry, "gpg-id") ||
                entry_has_extension(&our_entry, "gpg-id") ||
                entry_has_extension(&their_entry, "gpg-id")
            {
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

    pub fn conflicted_passwords(&self) -> Vec<ConflictedPassword> {
        self.conflicted_passwords.clone()
    }

    pub fn conflicted_gpg_ids(&self) -> Vec<ConflictedGpgId> {
        self.conflicted_gpg_ids.clone()
    }

    pub fn conflicted_plain_texts(&self) -> Vec<ConflictedPlainText> {
        self.conflicted_plain_texts.clone()
    }

    pub fn conflicted_binaries(&self) -> Vec<ConflictedBinary> {
        self.conflicted_binaries.clone()
    }

    pub fn finish(self) -> GitResult<()> {
        (self.finish_cb)(self.repository, self.maybe_index)
    }
}
