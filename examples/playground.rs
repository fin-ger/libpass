use std::path::PathBuf;

use pass::{Location, StoreBuilder, TraversalOrder};
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let repo_path = PathBuf::from("/home/fin/Development/playground/git/sample");
    let mut store = StoreBuilder::default()
        .location(Location::Manual(repo_path.clone()))
        .open()?;

    assert!(store.git().context("no git repo")?.config_valid());

    let mut root = store.show("./", TraversalOrder::LevelOrder)?
        .next()
        .context("Could not retrieve root directory")?
        .directory()
        .context("Could not retrieve root directory")?;

    let shuttle_bay = root.password_insertion("Shuttle Bay")
        .passphrase("0p3n-5354m3")
        .insert(&mut store)?;

    store.git().context("no git repo")?.add(&[shuttle_bay.path()])?;
    store.git().context("no git repo")?.commit("Add 'Shuttle Bay' password")?;

    let mut manufacturers = root.directory_insertion("Manufacturers")
        .insert(&mut store)?;

    let yoyodyne = manufacturers.password_insertion("Yoyodyne")
        .generator()
        .exclude_similar_characters(false)
        .length(20)
        .lowercase_letters(true)
        .numbers(true)
        .uppercase_letters(true)
        .spaces(true)
        .symbols(true)
        .strict(true)
        .generate(10)?
        .select(7)?
        .insert(&mut store)?;

    store.git().context("no git repo")?.add(&[yoyodyne.path()])?;
    store.git().context("no git repo")?.commit("Add 'Manufacturers/Yoyodyne' password")?;

    Ok(())
}
