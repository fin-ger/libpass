use anyhow::{Context, Result};
use pass::{Location, StoreBuilder, TraversalOrder, Sorting};

fn main() -> Result<()> {
    let mut store = StoreBuilder::default()
        .location(Location::Automatic)
        .open()?;

    assert!(!store.has_errors());
    assert!(store.git().context("no git repo")?.config_valid());

    store.sort(Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST);

    let entries = store.show(".", TraversalOrder::LevelOrder)?;
    for entry in entries {
        println!("{}", entry.path().display());
    }

    Ok(())
}
