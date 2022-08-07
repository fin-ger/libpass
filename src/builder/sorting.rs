use std::cmp::Ordering;

use bitflags::bitflags;
use id_tree::Node;

use crate::{PassNode, EntryKind};

bitflags! {
    pub struct Sorting: u8 {
        const NONE = 0b00000001;
        const ALPHABETICAL = 0b00000010;
        const DIRECTORIES_FIRST = 0b00000100;
    }
}

impl Sorting {
    pub(crate) fn cmp(&self, a: &Node<PassNode>, b: &Node<PassNode>) -> Ordering {
        let sort_dirs = self.contains(Sorting::DIRECTORIES_FIRST);
        let sort_alpha = self.contains(Sorting::ALPHABETICAL);

        if sort_dirs && a.data().kind() == EntryKind::Directory && b.data().kind() != EntryKind::Directory {
            Ordering::Less
        } else if sort_dirs && a.data().kind() != EntryKind::Directory && b.data().kind() == EntryKind::Directory {
            Ordering::Greater
        } else if sort_alpha {
            let a_low = a.data().name().to_lowercase();
            let b_low = b.data().name().to_lowercase();

            a_low.cmp(&b_low)
        } else {
            Ordering::Equal
        }
    }
}
