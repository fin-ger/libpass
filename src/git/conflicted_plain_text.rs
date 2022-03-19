use std::path::Path;

use super::{GitResult, conflict_resolver::{ConflictEntry, ConflictResolver}};

#[derive(Debug, Clone)]
pub struct ConflictedPlainText {
    ancestor: Option<ConflictEntry>,
    our: Option<ConflictEntry>,
    their: Option<ConflictEntry>,
    ancestor_content: Option<String>,
    our_content: Option<String>,
    their_content: Option<String>,
    is_resolved: bool,
}

impl ConflictedPlainText {
    pub(super) fn new(ancestor_entry: Option<ConflictEntry>, our_entry: Option<ConflictEntry>, their_entry: Option<ConflictEntry>) -> Option<Self> {
        let ancestor_content = if let Some(ancestor) = &ancestor_entry {
            Some(String::from_utf8(ancestor.content.to_vec()).ok()?)
        } else {
            None
        };
        let our_content = if let Some(our) = &our_entry {
            Some(String::from_utf8(our.content.to_vec()).ok()?)
        } else {
            None
        };
        let their_content = if let Some(their) = &their_entry {
            Some(String::from_utf8(their.content.to_vec()).ok()?)
        } else {
            None
        };

        Some(Self {
            ancestor_content,
            our_content,
            their_content,
            ancestor: ancestor_entry,
            our: our_entry,
            their: their_entry,
            is_resolved: false,
        })
    }

    pub fn ancestor_path(&self) -> Option<&Path> {
        if let Some(entry) = &self.ancestor {
            Some(&entry.path)
        } else {
            None
        }
    }

    pub fn our_path(&self) -> Option<&Path> {
        if let Some(entry) = &self.our {
            Some(&entry.path)
        } else {
            None
        }
    }

    pub fn their_path(&self) -> Option<&Path> {
        if let Some(entry) = &self.their {
            Some(&entry.path)
        } else {
            None
        }
    }

    pub fn ancestor_content(&self) -> Option<&str> {
        if let Some(content) = &self.ancestor_content {
            Some(content.as_str())
        } else {
            None
        }
    }

    pub fn our_content(&self) -> Option<&str> {
        if let Some(content) = &self.our_content {
            Some(content.as_str())
        } else {
            None
        }
    }

    pub fn their_content(&self) -> Option<&str> {
        if let Some(content) = &self.their_content {
            Some(content.as_str())
        } else {
            None
        }
    }

    pub fn resolve(&mut self, conflict_resolver: &mut ConflictResolver, resolved_content: Option<impl AsRef<str>>) -> GitResult<()> {
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
            let content = resolved_content.as_ref().as_bytes();
            let oid = conflict_resolver.repository.blob(content)?;
            conflict_entry.index_entry.file_size = content.len() as u32;
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
