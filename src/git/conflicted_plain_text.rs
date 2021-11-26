use std::{ffi::OsStr, path::Path};
use std::os::unix::ffi::OsStrExt;

use super::{GitResult, conflict_resolver::{ConflictEntry, ConflictResolver}};

#[derive(Debug)]
pub struct ConflictedPlainText {
    ancestor: ConflictEntry,
    our: ConflictEntry,
    their: ConflictEntry,
    ancestor_content: String,
    our_content: String,
    their_content: String,
    is_resolved: bool,
}

impl ConflictedPlainText {
    pub(super) fn new(ancestor_entry: ConflictEntry, our_entry: ConflictEntry, their_entry: ConflictEntry) -> Option<Self> {
        Some(Self {
            ancestor_content: String::from_utf8(ancestor_entry.content.to_vec()).ok()?,
            our_content: String::from_utf8(our_entry.content.to_vec()).ok()?,
            their_content: String::from_utf8(their_entry.content.to_vec()).ok()?,
            ancestor: ancestor_entry,
            our: our_entry,
            their: their_entry,
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

    pub fn ancestor_content(&self) -> &str {
        &self.ancestor_content
    }

    pub fn our_content(&self) -> &str {
        &self.our_content
    }

    pub fn their_content(&self) -> &str {
        &self.their_content
    }

    pub fn resolve(&mut self, conflict_resolver: &mut ConflictResolver, resolved_content: impl AsRef<str>, resolved_path: &Path) -> GitResult<()> {
        if self.is_resolved {
            return Err(git2::Error::new(git2::ErrorCode::Invalid, git2::ErrorClass::Merge, "Merge conflict already resolved"));
        }

        let index = conflict_resolver.maybe_index.as_mut()
            .expect("Conflict resolver has no index set when trying to resolve conflict");
        let entries = [&self.ancestor.index_entry, &self.our.index_entry, &self.their.index_entry];
        let index_entry = entries.iter()
            .find(|ie| Path::new(OsStr::from_bytes(&ie.path)) == resolved_path)
            .expect("No index entry matches path of resolved password");
        index.add_frombuffer(index_entry, resolved_content.as_ref().as_bytes())?;
        self.is_resolved = true;

        Ok(())
    }

    pub fn is_resolved(&self) -> bool {
        self.is_resolved
    }
}
