mod store;
mod directory;
mod password;
mod entries;
mod iterators;
mod git;
mod store_builder;

pub use store::*;
pub use directory::*;
pub use password::*;
pub use entries::*;
pub use iterators::*;
pub use git::*;
pub use store_builder::*;

#[cfg(test)]
mod tests {
    use crate::Sorting;

    #[test]
    fn sorting() {
        assert!(Sorting::NONE.bits() == 0, "Sorting::NONE is not 0");
        assert!(Sorting::ALPHABETICAL.bits() == 1, "Sorting::ALPHABETICAL is not 1");
        assert!(Sorting::DIRECTORIES_FIRST.bits() == 2, "Sorting::DIRECTORIES_FIRST is not 2");
        assert!((Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST).bits() == 3, "Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST is not 3");
    }
}
