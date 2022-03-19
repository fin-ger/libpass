use std::cmp::Ordering;

use bitflags::bitflags;
use id_tree::Node;

use crate::{PassNode, EntryKind};

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

        if sort_dirs && a.data().kind() == EntryKind::Directory && b.data().kind() != EntryKind::Directory {
            Ordering::Less
        } else if sort_dirs && a.data().kind() != EntryKind::Directory && b.data().kind() == EntryKind::Directory {
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

#[cfg(test)]
mod tests {
    use crate::Sorting;

    #[test]
    fn sorting() {
        assert!(Sorting::NONE.bits() == 0, "Sorting::NONE is not 0");
        assert!(
            Sorting::ALPHABETICAL.bits() == 1,
            "Sorting::ALPHABETICAL is not 1"
        );
        assert!(
            Sorting::DIRECTORIES_FIRST.bits() == 2,
            "Sorting::DIRECTORIES_FIRST is not 2"
        );
        assert!(
            (Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST).bits() == 3,
            "Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST is not 3"
        );
    }
}
