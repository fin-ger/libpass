use std::{ffi::OsStr, path::Path};
use std::os::unix::ffi::OsStrExt;

use crate::ConflictResolver;
use super::{GitResult, conflict_resolver::ConflictEntry};

#[derive(Debug)]
pub struct ConflictedBinary {
    ancestor: ConflictEntry,
    our: ConflictEntry,
    their: ConflictEntry,
    is_resolved: bool,
}

impl ConflictedBinary {
    pub(super) fn new(ancestor: ConflictEntry, our: ConflictEntry, their: ConflictEntry) -> Self {
        Self {
            ancestor,
            our,
            their,
            is_resolved: false,
        }
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

    pub fn ancestor_content(&self) -> &[u8] {
        &self.ancestor.content
    }

    pub fn our_content(&self) -> &[u8] {
        &self.our.content
    }

    pub fn their_content(&self) -> &[u8] {
        &self.their.content
    }

    pub fn resolve(&mut self, conflict_resolver: &mut ConflictResolver, resolved_content: &[u8], resolved_path: &Path) -> GitResult<()> {
        if self.is_resolved {
            return Err(git2::Error::new(git2::ErrorCode::Invalid, git2::ErrorClass::Merge, "Merge conflict already resolved"));
        }

        let index = conflict_resolver.maybe_index.as_mut()
            .expect("Conflict resolver has no index set when trying to resolve conflict");
        let entries = [&self.ancestor.index_entry, &self.our.index_entry, &self.their.index_entry];
        let index_entry = entries.iter()
            .find(|ie| Path::new(OsStr::from_bytes(&ie.path)) == resolved_path)
            .expect("No index entry matches path of resolved password");
        index.add_frombuffer(index_entry, resolved_content)?;
        self.is_resolved = true;

        Ok(())
    }

    pub fn is_resolved(&self) -> bool {
        self.is_resolved
    }
}
