use std::{env, fs, io};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::iter::Skip;

use thiserror::Error;
use id_tree::{Tree, Node, NodeId, InsertBehavior, LevelOrderTraversalIds, PostOrderTraversalIds, PreOrderTraversalIds};
use directories::BaseDirs;
use bitflags::bitflags;

use crate::{Directory, StoreReadError, IntoStoreReadError};

pub enum Location {
    /// $PASSWORD_STORE_DIR or if not set ~/.password-store
    Automatic,
    /// Override the path
    Manual(PathBuf),
}

#[derive(Clone)]
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

bitflags! {
    pub struct Sorting: u8 {
        const NONE = 0;
        const ALPHABETICAL = 1;
        const DIRECTORIES_FIRST = 2;
    }
}

impl Sorting {
    pub(crate) fn cmp(&self, a: &Node<PassNode>, b: &Node<PassNode>) -> Ordering {
        let sort_dirs = self.contains(Sorting::DIRECTORIES_FIRST);
        let sort_alpha = self.contains(Sorting::ALPHABETICAL);

        if sort_dirs && a.data().is_dir() && !b.data().is_dir() {
            Ordering::Less
        } else if sort_dirs && !a.data().is_dir() && b.data().is_dir() {
            Ordering::Greater
        } else if sort_alpha {
            let a_low = a.data().name().to_lowercase();
            let b_low = b.data().name().to_lowercase();

            a_low.cmp(&b_low)
        } else {
            Ordering::Less
        }
    }
}

pub struct Store {
    path: PathBuf,
    tree: Tree<PassNode>,
    errors: Vec<StoreReadError>,
}

impl Store {
    pub fn init(_location: Location, _key_id: String) -> Result<Self, StoreError> {
        unimplemented!();
    }

    pub fn open(location: Location) -> Result<Self, StoreError> {
        let path = match location {
            Location::Automatic => {
                env::var("PASSWORD_STORE_DIR")
                    .with_store_error()
                    .map(|password_store_dir| {
                        Path::new(&password_store_dir).to_owned()
                    })
                    .or_else(|e| {
                        BaseDirs::new()
                            .map(|base_dirs| base_dirs.home_dir().join(".password-store"))
                            .ok_or(e)
                            .with_store_error()
                    })?
            },
            Location::Manual(path) => path,
        };

        let metadata = path.metadata().with_store_error()?;
        if !metadata.is_dir() {
            return Err(StoreError::NoDirectory(path));
        }

        let tree = Tree::new();
        let mut me = Self {
            path,
            tree,
            errors: Vec::new(),
        };
        me.load_passwords();

        Ok(me)
    }

    pub fn with_sorting(mut self, sorting: Sorting) -> Self {
        self.sort(sorting);
        self
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> StoreReadErrors {
        StoreReadErrors::new(&self.errors)
    }

    pub fn sort(&mut self, sorting: Sorting) {
        let root_id = self.tree.root_node_id()
            .expect("Cannot find root node in internal tree").clone();
        let level_order = self.tree.traverse_level_order_ids(&root_id)
            .expect("Failed to traverse internal tree")
            .collect::<Vec<_>>();

        for node_id in level_order {
            self.tree.sort_children_by(&node_id, |a, b| sorting.cmp(a, b))
                .expect("Failed to sort internal tree");
        }
    }

    fn load_passwords(&mut self) {
        self.tree = Tree::new();
        let root_id = self.tree.insert(
            Node::new(PassNode::Directory {
                name: ".".into(),
                path: self.path.clone(),
            }),
            InsertBehavior::AsRoot,
        ).expect("Failed to create internal tree representation");

        self.load_passwords_from_dir(&self.path.clone(), &root_id);
    }

    fn is_special_entry(path: &Path) -> bool {
        match path.file_name().unwrap_or("..".as_ref()).to_string_lossy().as_ref() {
            ".git" | ".gitattributes" | ".gpg-id" => true,
            _ => false,
        }
    }

    fn load_passwords_from_dir(&mut self, dir: &Path, parent: &NodeId) {
        let (read_dir, mut errors) = match fs::read_dir(dir).with_store_read_error(dir) {
            Ok(read_dir) => {
                let (mut read_dir_res, mut errors_res): (Vec<_>, Vec<_>) = read_dir.partition(Result::is_ok);
                (
                    read_dir_res.drain(..).map(|res| res.unwrap()).collect(),
                    errors_res.drain(..).map(|res| res.with_store_read_error(dir).unwrap_err()).collect(),
                )
            },
            Err(err) => (vec![], vec![err]),
        };

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
                        self.load_passwords_from_dir(&path, &subdir);
                    },
                    None => {
                        errors.push(StoreReadError::name_error(&path));
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
                        errors.push(StoreReadError::name_error(&path));
                    },
                }
            }
        }

        self.errors.extend(errors);
    }

    pub fn content(&self) -> Directory {
        let root_id = self.tree.root_node_id()
            .expect("Failed to get root node of internal tree");
        Directory::new(".", &self.path, &self.tree, root_id)
    }

    pub fn traverse_recursive(&self, order: TraversalOrder) -> RecursiveTraversal {
        RecursiveTraversal::new(&self.tree, order)
    }

    pub fn location(&self) -> &Path {
        &self.path
    }
}

pub enum TraversalOrder {
    LevelOrder,
    PostOrder,
    PreOrder,
}

enum InnerRecursiveTraversal<'a> {
    LevelOrder(Skip<LevelOrderTraversalIds<'a, PassNode>>),
    PostOrder(Skip<PostOrderTraversalIds>),
    PreOrder(Skip<PreOrderTraversalIds<'a, PassNode>>),
}

use crate::DirectoryEntry;

pub struct RecursiveTraversal<'a> {
    iter: InnerRecursiveTraversal<'a>,
    tree: &'a Tree<PassNode>,
}

impl<'a> RecursiveTraversal<'a> {
    fn new(tree: &'a Tree<PassNode>, order: TraversalOrder) -> Self {
        let root_id = tree.root_node_id()
            .expect("Failed to retrieve root node of internal tree")
            .clone();

        let iter = match order {
            TraversalOrder::LevelOrder => InnerRecursiveTraversal::LevelOrder(
                tree.traverse_level_order_ids(&root_id)
                    .expect("Failed to traverse level order on the internal tree")
                    .skip(1)
            ),
            TraversalOrder::PostOrder => InnerRecursiveTraversal::PostOrder(
                tree.traverse_post_order_ids(&root_id)
                    .expect("Failed to traverse post order on the internal tree")
                    .skip(1)
            ),
            TraversalOrder::PreOrder => InnerRecursiveTraversal::PreOrder(
                tree.traverse_pre_order_ids(&root_id)
                    .expect("Failed to traverse pre order on the internal tree")
                    .skip(1)
            ),
        };

        Self {
            iter,
            tree,
        }
    }
}

impl<'a> Iterator for RecursiveTraversal<'a> {
    type Item = DirectoryEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node_id = match self.iter {
            InnerRecursiveTraversal::LevelOrder(ref mut t) => t.next(),
            InnerRecursiveTraversal::PostOrder(ref mut t) => t.next(),
            InnerRecursiveTraversal::PreOrder(ref mut t) => t.next(),
        }?;

        Some(DirectoryEntry::new(node_id, self.tree))
    }
}

pub struct StoreReadErrors<'a> {
    iter: std::slice::Iter<'a, StoreReadError>,
}

impl<'a> StoreReadErrors<'a> {
    fn new(errors: &'a Vec<StoreReadError>) -> Self {
        Self {
            iter: errors.into_iter()
        }
    }
}

impl<'a> Iterator for StoreReadErrors<'a> {
    type Item = &'a StoreReadError;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Could not open password store")]
    Io(#[source] io::Error),
    #[error("Password store path is not a directory: {0}")]
    NoDirectory(PathBuf),
    #[error("PASSWORD_STORE_DIR environment variable is not set")]
    EnvVar(#[source] env::VarError),
    #[error("Cannot find home directory for current user")]
    NoHome(#[source] Box<StoreError>),
}

trait IntoStoreError<T> {
    fn with_store_error(self: Self) -> Result<T, StoreError>;
}

impl<T> IntoStoreError<T> for Result<T, env::VarError> {
    fn with_store_error(self: Self) -> Result<T, StoreError> {
        self.map_err(|err| {
            StoreError::EnvVar(err)
        })
    }
}

impl<T> IntoStoreError<T> for Result<T, StoreError> {
    fn with_store_error(self: Self) -> Result<T, StoreError> {
        self.map_err(|err| {
            StoreError::NoHome(Box::new(err))
        })
    }
}

impl<T> IntoStoreError<T> for Result<T, io::Error> {
    fn with_store_error(self: Self) -> Result<T, StoreError> {
        self.map_err(|err| {
            StoreError::Io(err)
        })
    }
}
