use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) enum PassNode {
    Directory { name: String, path: PathBuf },
    Password { name: String, path: PathBuf },
    NormalFile { name: String, path: PathBuf },
}

#[derive(Debug, PartialEq)]
pub enum EntryKind {
    Password,
    Directory,
    NormalFile,
}

impl PassNode {
    pub fn kind(&self) -> EntryKind {
        match self {
            PassNode::Password { .. } => EntryKind::Password,
            PassNode::Directory { .. } => EntryKind::Directory,
            PassNode::NormalFile { .. } => EntryKind::NormalFile,
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Directory { ref name, .. } => name,
            Self::Password { ref name, .. } => name,
            Self::NormalFile { ref name, .. } => name,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Directory { ref path, .. } => path,
            Self::Password { ref path, .. } => path,
            Self::NormalFile { ref path, .. } => path,
        }
    }
}
