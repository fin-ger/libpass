mod store;
mod directory;
mod password;
mod entries;
mod iterators;
mod git;

pub use store::*;
pub use directory::*;
pub use password::*;
pub use entries::*;
pub use iterators::*;
pub use git::*;

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use crate::{Store, Location, Sorting, Directory};

    fn print_dir(dir: &Directory<'_>) {
        println!("Passwords:");
        for password in dir.passwords() {
            println!("  {}: {}", password.path().display(), password.name());
        }

        println!("Directories:");
        for dir in dir.directories() {
            println!("  {}: {}", dir.path().display(), dir.name());
        }

        println!("Entries:");
        for entry in dir.entries() {
            let kind = if entry.is_password() { "PW" } else { "DIR" };
            println!("  {} {}: {}", kind, entry.path().display(), entry.name());
        }

        println!();

        for dir in dir.directories() {
            print_dir(&dir);
        }
    }

    fn print_errors(store: &Store) {
        for err in store.errors() {
            println!("{:?}", err);
        }
    }

    #[test]
    fn no_sorting() -> Result<()> {
        let store = Store::open(Location::Automatic)?
            .with_sorting(Sorting::NONE);
        print_errors(&store);
        assert!(!store.has_errors());
        let content = store.content();
        println!(">>> no sorting <<<");
        print_dir(&content);
        Ok(())
    }

    #[test]
    fn sorting() {
        assert!(Sorting::NONE.bits() == 0, "Sorting::NONE is not 0");
        assert!(Sorting::ALPHABETICAL.bits() == 1, "Sorting::ALPHABETICAL is not 1");
        assert!(Sorting::DIRECTORIES_FIRST.bits() == 2, "Sorting::DIRECTORIES_FIRST is not 2");
        assert!((Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST).bits() == 3, "Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST is not 3");
    }
}
