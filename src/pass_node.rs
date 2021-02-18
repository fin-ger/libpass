use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) enum PassNode {
    Directory { name: String, path: PathBuf },
    Password { name: String, path: PathBuf },
}

impl PassNode {
    pub(crate) fn is_dir(&self) -> bool {
        if let Self::Directory { .. } = self {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_password(&self) -> bool {
        !self.is_dir()
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Directory { ref name, .. } => name,
            Self::Password { ref name, .. } => name,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Directory { ref path, .. } => path,
            Self::Password { ref path, .. } => path,
        }
    }
}
