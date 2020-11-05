use id_tree::{Tree, Node, NodeId, InsertBehavior, LevelOrderTraversalIds, PostOrderTraversalIds, PreOrderTraversalIds};
use anyhow::{anyhow, Context, Result, Error};
use directories::BaseDirs;
use bitflags::bitflags;

use std::{env, fs};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use crate::Directory;

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

    pub fn with_sorting(mut self, sorting: Sorting) -> Self {
        self.sort(sorting);
        self
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

    pub fn content(&self) -> Directory {
        let root_id = self.tree.root_node_id()
            .expect("Failed to get root node of internal tree");
        Directory::new(".", &self.path, &self.tree, root_id)
    }

    pub fn traverse_recursive(&self, order: TraversalOrder) -> RecursiveTraversal {
        RecursiveTraversal::new(&self.tree, order)
    }
}

pub enum TraversalOrder {
    LevelOrder,
    PostOrder,
    PreOrder,
}

enum InnerRecursiveTraversal<'a> {
    LevelOrder(LevelOrderTraversalIds<'a, PassNode>),
    PostOrder(PostOrderTraversalIds),
    PreOrder(PreOrderTraversalIds<'a, PassNode>),
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
            ),
            TraversalOrder::PostOrder => InnerRecursiveTraversal::PostOrder(
                tree.traverse_post_order_ids(&root_id)
                    .expect("Failed to traverse post order on the internal tree")
            ),
            TraversalOrder::PreOrder => InnerRecursiveTraversal::PreOrder(
                tree.traverse_pre_order_ids(&root_id)
                    .expect("Failed to traverse pre order on the internal tree")
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
