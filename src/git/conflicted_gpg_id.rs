use std::{collections::HashSet, ffi::OsStr, path::Path};
use std::os::unix::ffi::OsStrExt;

use crate::{ConflictResolver, GpgKeyId};

use super::{GitResult, conflict_resolver::ConflictEntry};

#[derive(Debug, Clone)]
pub struct ConflictedGpgId {
    ancestor: ConflictEntry,
    our: ConflictEntry,
    their: ConflictEntry,
    ancestor_key_ids: HashSet<GpgKeyId>,
    our_key_ids: HashSet<GpgKeyId>,
    their_key_ids: HashSet<GpgKeyId>,
    is_resolved: bool,
}

impl ConflictedGpgId {
    pub(super) fn new(ancestor: ConflictEntry, our: ConflictEntry, their: ConflictEntry) -> Option<Self> {
        let ancestor_key_ids = String::from_utf8(ancestor.content.to_vec()).ok()?
            .lines()
            .map(|key_id| GpgKeyId::new(key_id).ok())
            .collect::<Option<HashSet<_>>>()?;
        let our_key_ids = String::from_utf8(our.content.to_vec()).ok()?
            .lines()
            .map(|key_id| GpgKeyId::new(key_id).ok())
            .collect::<Option<HashSet<_>>>()?;
        let their_key_ids = String::from_utf8(their.content.to_vec()).ok()?
            .lines()
            .map(|key_id| GpgKeyId::new(key_id).ok())
            .collect::<Option<HashSet<_>>>()?;

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

    pub fn ancestor_path(&self) -> &Path {
        &self.ancestor.path
    }

    pub fn our_path(&self) -> &Path {
        &self.our.path
    }

    pub fn their_path(&self) -> &Path {
        &self.their.path
    }

    pub fn ancestor_key_ids(&self) -> &HashSet<GpgKeyId> {
        &self.ancestor_key_ids
    }

    pub fn our_key_ids(&self) -> &HashSet<GpgKeyId> {
        &self.our_key_ids
    }

    pub fn their_key_ids(&self) -> &HashSet<GpgKeyId> {
        &self.their_key_ids
    }

    pub fn resolve(&mut self, conflict_resolver: &mut ConflictResolver, resolved_gpg_ids: &HashSet<GpgKeyId>, resolved_path: &Path) -> GitResult<()> {
        if self.is_resolved {
            return Err(git2::Error::new(git2::ErrorCode::Invalid, git2::ErrorClass::Merge, "Merge conflict already resolved"));
        }

        let index = conflict_resolver.maybe_index.as_mut()
            .expect("Conflict resolver has no index set when trying to resolve conflict");
        // FIXME: re-encrypt all passwords this gpg-id controls
        let resolved_content = resolved_gpg_ids.iter()
            .map(|key| {
                key.get_key()
                    .id().expect("GPG key id not valid utf-8")
                    .to_owned()
            })
            .reduce(|res, key_id| format!("{}\n{}", res, key_id))
            .unwrap_or(String::new());
        let entries = [&self.ancestor.index_entry, &self.our.index_entry, &self.their.index_entry];
        let index_entry = entries.iter()
            .find(|ie| Path::new(OsStr::from_bytes(&ie.path)) == resolved_path)
            .expect("No index entry matches path of resolved password");
        index.add_frombuffer(index_entry, resolved_content.as_bytes())?;
        self.is_resolved = true;

        Ok(())
    }

    pub fn is_resolved(&self) -> bool {
        self.is_resolved
    }
}
