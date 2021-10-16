use std::{fs::File, io::Write, path::{Path, PathBuf}};

use pass::{Location, StoreBuilder};
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let repo_path = PathBuf::from("/home/fin/Development/playground/git/sample");
    let mut store = StoreBuilder::default()
        .location(Location::Manual(repo_path.clone()))
        .open()?;

    assert!(store.git().context("no git repo")?.config_valid());

    let mut first_file = File::create(repo_path.join("first_file.txt"))?;
    first_file.write_all("This is the content of the first file".as_bytes())?;

    store.git().context("no git repo")?.add(&[&Path::new("first_file.txt")])?;
    store.git().context("no git repo")?.commit("First file added")?;

    let mut second_file = File::create(repo_path.join("second_file.txt"))?;
    second_file.write_all("This is the content of the second file".as_bytes())?;

    store.git().context("no git repo")?.add(&[&Path::new("second_file.txt")])?;
    store.git().context("no git repo")?.commit("Second file added")?;

    Ok(())
}
