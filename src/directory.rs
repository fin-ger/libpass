use id_tree::{Tree, NodeId};

use std::cmp::Ordering;
use std::path::Path;

use crate::{PassNode, Sorting, Passwords, Directories, Entries};

pub struct Directory<'a> {
    name: &'a str,
    path: &'a Path,
    tree: &'a Tree<PassNode>,
    entries: Vec<&'a NodeId>,
    sorting: Sorting,
}

impl<'a> Directory<'a> {
    pub(crate) fn new(
        name: &'a str,
        path: &'a Path,
        tree: &'a Tree<PassNode>,
        node: &NodeId,
        sorting: Sorting,
    ) -> Self {
        let mut entries: Vec<_> = tree.children_ids(node)
            .expect("Failed to read directory entries from internal tree")
            .collect();
        let sort_dirs = sorting.contains(Sorting::DIRECTORIES_FIRST);
        let sort_alpha = sorting.contains(Sorting::ALPHABETICAL);

        entries.sort_by(|a, b| {
            let a = tree.get(a).expect("Failed to find node in internal tree");
            let b = tree.get(b).expect("Failed to find node in internal tree");
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
        });

        Self {
            name,
            path,
            tree,
            entries,
            sorting,
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn path(&self) -> &'a Path {
        self.path
    }

    pub fn passwords(&self) -> Passwords {
        Passwords::new(self.entries())
    }

    pub fn directories(&self) -> Directories {
        Directories::new(self.entries())
    }

    pub fn entries(&self) -> Entries {
        Entries::new(self.tree, self.sorting.clone(), self.entries.iter())
    }
}
