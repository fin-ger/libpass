use std::{collections::HashSet, path::Path};

use crate::{ConflictResolver, GpgKeyId};

use super::{GitResult, conflict_resolver::ConflictEntry};

#[derive(Debug, Clone)]
pub struct ConflictedGpgId {
    ancestor: Option<ConflictEntry>,
    our: Option<ConflictEntry>,
    their: Option<ConflictEntry>,
    ancestor_key_ids: Option<HashSet<GpgKeyId>>,
    our_key_ids: Option<HashSet<GpgKeyId>>,
    their_key_ids: Option<HashSet<GpgKeyId>>,
    is_resolved: bool,
}

impl ConflictedGpgId {
    pub(super) fn new(ancestor: Option<ConflictEntry>, our: Option<ConflictEntry>, their: Option<ConflictEntry>) -> Option<Self> {
        let ancestor_key_ids = if let Some(ancestor) = &ancestor {
            Some(
                String::from_utf8(ancestor.content.to_vec()).ok()?
                    .lines()
                    .map(|key_id| GpgKeyId::new(key_id).ok())
                    .collect::<Option<HashSet<_>>>()?
            )
        } else { None };

        let our_key_ids = if let Some(our) = &our {
            Some(
                String::from_utf8(our.content.to_vec()).ok()?
                    .lines()
                    .map(|key_id| GpgKeyId::new(key_id).ok())
                    .collect::<Option<HashSet<_>>>()?
            )
        } else { None };

        let their_key_ids = if let Some(their) = &their {
            Some(
                String::from_utf8(their.content.to_vec()).ok()?
                    .lines()
                    .map(|key_id| GpgKeyId::new(key_id).ok())
                    .collect::<Option<HashSet<_>>>()?
            )
        } else { None };

        Some(Self {
            ancestor,
            our,
            their,
            ancestor_key_ids,
            our_key_ids,
            their_key_ids,
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
        if let Some(our) = &self.ancestor {
            Some(&our.path)
        } else {
            None
        }
    }

    pub fn their_path(&self) -> Option<&Path> {
        if let Some(their) = &self.ancestor {
            Some(&their.path)
        } else {
            None
        }
    }

    pub fn ancestor_key_ids(&self) -> Option<&HashSet<GpgKeyId>> {
        self.ancestor_key_ids.as_ref()
    }

    pub fn our_key_ids(&self) -> Option<&HashSet<GpgKeyId>> {
        self.our_key_ids.as_ref()
    }

    pub fn their_key_ids(&self) -> Option<&HashSet<GpgKeyId>> {
        self.their_key_ids.as_ref()
    }

    pub fn resolve(&mut self, conflict_resolver: &mut ConflictResolver, resolved_gpg_ids: Option<&HashSet<GpgKeyId>>) -> GitResult<()> {
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

        if let Some(resolved_gpg_ids) = resolved_gpg_ids {
            // passwords are not re-encrypted, as there might exist passwords
            // which contain changes unrelated to the gpg-id change. We don't
            // want to overwrite those changes and therefore leave re-encrypting
            // up to the user. Usually, this is intuitively done, as a gpg-id
            // change commit also contains already re-encrypted passwords.
            // Therefore, all passwords will have a conflict and must therefore
            // be handled by the user.
            let resolved_content = resolved_gpg_ids.iter()
                .map(|key| {
                    key.id().to_owned()
                })
                .reduce(|res, key_id| format!("{}\n{}", res, key_id))
                .unwrap_or(String::new());
            let content = resolved_content.as_bytes();
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
