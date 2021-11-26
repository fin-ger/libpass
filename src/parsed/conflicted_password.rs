use std::{ffi::OsStr, fmt, path::{Path, PathBuf}};
use std::os::unix::ffi::OsStrExt;

use crate::{ConflictResolver, IntoStoreError, Position, StoreError};

#[derive(Debug, Clone)]
pub struct ConflictedDecryptedPassword;

impl ConflictedDecryptedPassword {
}
