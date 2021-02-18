use id_tree::{ChildrenIds, Tree};

use crate::{Directory, Entry, PassNode, Password};

pub struct Entries<'a> {
    tree: &'a Tree<PassNode>,
    iter: ChildrenIds<'a>,
}

impl<'a> Entries<'a> {
    pub(crate) fn new(tree: &'a Tree<PassNode>, iter: ChildrenIds<'a>) -> Self {
        Self { tree, iter }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|id| Entry::new(id.clone(), self.tree))
    }
}

pub struct Passwords<'a> {
    entries: Entries<'a>,
}

impl<'a> Passwords<'a> {
    pub(crate) fn new(entries: Entries<'a>) -> Self {
        Self { entries }
    }
}

impl<'a> Iterator for Passwords<'a> {
    type Item = Password<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pw = None;
        while pw.is_none() {
            pw = self.entries.next()?.password()
        }

        pw
    }
}

pub struct Directories<'a> {
    entries: Entries<'a>,
}

impl<'a> Directories<'a> {
    pub(crate) fn new(entries: Entries<'a>) -> Self {
        Self { entries }
    }
}

impl<'a> Iterator for Directories<'a> {
    type Item = Directory<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut dir = None;
        while dir.is_none() {
            dir = self.entries.next()?.directory()
        }

        dir
    }
}
