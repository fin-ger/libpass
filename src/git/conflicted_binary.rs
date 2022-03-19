use std::path::Path;

use crate::ConflictResolver;
use super::{GitResult, conflict_resolver::ConflictEntry};

#[derive(Debug, Clone)]
pub struct ConflictedBinary {
    ancestor: Option<ConflictEntry>,
    our: Option<ConflictEntry>,
    their: Option<ConflictEntry>,
    is_resolved: bool,
}

impl ConflictedBinary {
    pub(super) fn new(ancestor: Option<ConflictEntry>, our: Option<ConflictEntry>, their: Option<ConflictEntry>) -> Self {
        Self {
            ancestor,
            our,
            their,
            is_resolved: false,
        }
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

    pub fn ancestor_content(&self) -> Option<&[u8]> {
        if let Some(ancestor) = &self.ancestor {
            Some(&ancestor.content)
        } else {
            None
        }
    }

    pub fn our_content(&self) -> Option<&[u8]> {
        if let Some(our) = &self.our {
            Some(&our.content)
        } else {
            None
        }
    }

    pub fn their_content(&self) -> Option<&[u8]> {
        if let Some(their) = &self.their {
            Some(&their.content)
        } else {
            None
        }
    }

    pub fn resolve(&mut self, conflict_resolver: &mut ConflictResolver, resolved_content: Option<&[u8]>) -> GitResult<()> {
        if self.is_resolved {
            return Err(git2::Error::new(git2::ErrorCode::Invalid, git2::ErrorClass::Merge, "Merge conflict already resolved"));
        }

        let index = conflict_resolver.maybe_index.as_mut()
            .expect("Conflict resolver has no index set when trying to resolve conflict");
        let mut entries = self.our.iter()
            .chain(self.ancestor.iter())
            .chain(self.their.iter());
        let mut conflict_entry = entries
            .next()
            .expect("No index entry available for conflict resolution. So it finally happened... Couldn't produce a test case triggering this behavior and libgit2 docs say nothing about it. Please report it in libpass's issue tracker!")
            .clone();

        if let Some(resolved_content) = resolved_content {
            let oid = conflict_resolver.repository.blob(resolved_content)?;
            conflict_entry.index_entry.file_size = resolved_content.len() as u32;
            conflict_entry.index_entry.id = oid;

            index.add(&conflict_entry.index_entry)?;
        } else {
            // stage is ANY (-1) to remove it from ancestor, ours, and theirs
            index.remove(&conflict_entry.path, -1)?;
        }

        index.conflict_remove(&conflict_entry.path)?;

        self.is_resolved = true;

        Ok(())
    }

    pub fn is_resolved(&self) -> bool {
        self.is_resolved
    }
}
