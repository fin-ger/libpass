use id_tree::{Tree, Node, NodeId, InsertBehavior};
use anyhow::{anyhow, Context, Result, Error};
use directories::BaseDirs;
use bitflags::bitflags;

use std::{env, fs};
use std::path::{Path, PathBuf};

use crate::Directory;

pub enum Location {
    /// $PASSWORD_STORE_DIR or if not set ~/.password-store
    Automatic,
    /// Override the path
    Manual(PathBuf),
}

pub(crate) enum PassNode {
    Directory {
        name: String,
        path: PathBuf,
    },
    Password {
        name: String,
        path: PathBuf,
    },
}

impl PassNode {
    pub(crate) fn is_dir(&self) -> bool {
        if let Self::Directory { .. } = self {
            true
        } else {
            false
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Directory { ref name, .. } => name,
            Self::Password { ref name, .. } => name,
        }
    }
}

bitflags! {
    pub struct Sorting: u8 {
        const NONE = 0;
        const ALPHABETICAL = 1;
        const DIRECTORIES_FIRST = 2;
    }
}

pub struct Store {
    path: PathBuf,
    tree: Tree<PassNode>,
}

impl Store {
    pub fn open(location: Location) -> Result<Self> {
        let path = match location {
            Location::Automatic => {
                env::var("PASSWORD_STORE_DIR")
                    .context("PASSWORD_STORE_DIR environment variable is not set")
                    .map(|password_store_dir| {
                        Path::new(&password_store_dir).to_owned()
                    })
                    .or_else(|e| {
                        BaseDirs::new()
                            .map(|base_dirs| base_dirs.home_dir().join(".password-store"))
                            .ok_or(e)
                            .context("Cannot find home directory for current user")
                    })?
            },
            Location::Manual(path) => path,
        };

        let tree = Tree::new();
        let mut me = Self {
            path,
            tree,
        };
        let _errors = me.load_passwords()?; // TODO: propagate errors

        Ok(me)
    }

    fn load_passwords(&mut self) -> Result<Vec<Error>> {
        self.tree = Tree::new();
        let root_id = self.tree.insert(
            Node::new(PassNode::Directory {
                name: ".".into(),
                path: self.path.clone(),
            }),
            InsertBehavior::AsRoot,
        ).expect("Failed to create internal tree representation");

        self.load_passwords_from_dir(&self.path.clone(), &root_id)
    }

    fn is_special_entry(path: &Path) -> bool {
        match path.file_name().unwrap_or("..".as_ref()).to_string_lossy().as_ref() {
            ".git" | ".gitattributes" | ".gpg-id" => true,
            _ => false,
        }
    }

    fn load_passwords_from_dir(&mut self, dir: &Path, parent: &NodeId) -> Result<Vec<Error>> {
        let (mut read_dir, mut errors): (Vec<_>, Vec<_>) = fs::read_dir(dir)
            .context(format!("Cannot open directory: {}", dir.display()))?
            .partition(Result::is_ok);

        let read_dir = read_dir.drain(..).map(|res| res.unwrap());
        let mut errors: Vec<_> = errors
            .drain(..)
            .map(|res| res.unwrap_err().into())
            .collect();

        for entry in read_dir {
            let path = entry.path();

            if path.is_dir() && !Self::is_special_entry(&path) {
                match path.file_name() {
                    Some(name) => {
                        let subdir = self.tree.insert(
                            Node::new(PassNode::Directory {
                                name: name.to_string_lossy().to_string(),
                                path: path.clone(),
                            }),
                            InsertBehavior::UnderNode(parent),
                        ).expect("Failed to insert directory into internal tree");
                        self.load_passwords_from_dir(&path, &subdir)?;
                    },
                    None => {
                        errors.push(anyhow!("Cannot get directory name for '{}'", path.display()));
                    },
                }
            } else if !Self::is_special_entry(&path) {
                match path.file_stem() {
                    Some(name) => {
                        let _pw_id = self.tree.insert(
                            Node::new(PassNode::Password {
                                name: name.to_string_lossy().to_string(),
                                path,
                            }),
                            InsertBehavior::UnderNode(parent),
                        ).expect("Failed to insert password into internal tree");
                    },
                    None => {
                        errors.push(anyhow!("Cannot get password name for '{}'", path.display()));
                    },
                }
            }
        }

        Ok(errors)
    }

    pub fn content(&self, sorting: Sorting) -> Directory {
        let root_id = self.tree.root_node_id()
            .expect("Failed to get root node of internal tree");
        Directory::new(".", &self.path, &self.tree, root_id, sorting)
    }
}
