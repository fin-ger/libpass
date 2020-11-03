use id_tree::{Tree, NodeId};

use std::slice;
use std::path::Path;

use crate::{Password, Directory, Sorting, PassNode};

pub enum DirectoryEntry<'a> {
    Password(Password<'a>),
    Directory(Directory<'a>),
}

impl<'a> DirectoryEntry<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            DirectoryEntry::Password(pw) => pw.name(),
            DirectoryEntry::Directory(dir) => dir.name(),
        }
    }

    pub fn path(&self) -> &'a Path {
        match self {
            DirectoryEntry::Password(pw) => pw.path(),
            DirectoryEntry::Directory(dir) => dir.path(),
        }
    }

    pub fn is_dir(&self) -> bool {
        if let DirectoryEntry::Directory(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_password(&self) -> bool {
        !self.is_dir()
    }

    pub fn password(self) -> Option<Password<'a>> {
        if let DirectoryEntry::Password(pw) = self {
            Some(pw)
        } else {
            None
        }
    }

    pub fn directory(self) -> Option<Directory<'a>> {
        if let DirectoryEntry::Directory(dir) = self {
            Some(dir)
        } else {
            None
        }
    }
}

pub struct Entries<'a> {
    tree: &'a Tree<PassNode>,
    sorting: Sorting,
    iter: slice::Iter<'a, &'a NodeId>,
}

impl<'a> Entries<'a> {
    pub(crate) fn new(
        tree: &'a Tree<PassNode>,
        sorting: Sorting,
        iter: slice::Iter<'a, &'a NodeId>,
    ) -> Self {
        Self {
            tree,
            sorting,
            iter,
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = DirectoryEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.iter.next()?;
        let node = self.tree.get(id).expect("Failed to find node id in internal tree");

        Some(match node.data() {
            PassNode::Password { name, path } => {
                DirectoryEntry::Password(Password::new(name, path))
            },
            PassNode::Directory { name, path } => {
                DirectoryEntry::Directory(Directory::new(name, path, self.tree, id, self.sorting.clone()))
            },
        })
    }
}
