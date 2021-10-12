use pass::StoreBuilder;
use anyhow::Result;

fn main() -> Result<()> {
    let store = StoreBuilder::default().open()?;
    println!("{}", store.git().config_valid());

    Ok(())
}
