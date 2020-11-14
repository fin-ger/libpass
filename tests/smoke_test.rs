mod common;

use anyhow::Result;
use pass::{Store, Location, Sorting, TraversalOrder};

const DIR: bool = true;
const PW: bool = false;

#[test]
fn smoke() -> Result<()> {
    let setup = common::setup("smoke", &["smoke"]);
    let store = Store::open(Location::Automatic)?
        .with_sorting(Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST);
    assert!(!store.has_errors());

    let expected = [
        (DIR, "Entertainment"),
        (DIR, "Manufacturers"),
        (PW, "Phone.gpg"),
        (DIR, "Entertainment/Holo Deck"),
        (PW, "Manufacturers/Sokor.gpg"),
        (PW, "Manufacturers/StrutCo.gpg"),
        (PW, "Manufacturers/Yoyodyne.gpg"),
        (PW, "Entertainment/Holo Deck/Broht & Forrester.gpg"),
    ];

    for (ref entry, (is_dir, expected)) in store.traverse_recursive(TraversalOrder::LevelOrder).zip(&expected) {
        if *is_dir {
            assert!(entry.is_dir(), format!("{} is not a directory", entry.path().display()));
        } else {
            assert!(entry.is_password(), format!("{} is not a password", entry.path().display()));
        }
        assert!(entry.path().ends_with(expected), format!("{} is not {}", entry.path().display(), expected));
    }

    common::teardown(setup);
    Ok(())
}
