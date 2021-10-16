use anyhow::{Context, Result};
use pass::{StoreBuilder, TraversalOrder};
use rand::seq::SliceRandom;

fn main() -> Result<()> {
    let store = StoreBuilder::default().open()?;
    assert!(!store.has_errors());
    let mut root_password = store
        .show(
            "./Enterprise/Self-Destruct Sequence.gpg",
            TraversalOrder::PostOrder,
        )?
        .next()
        .context("Password not found")?
        .password()
        .context("Entry is not a password")?
        .decrypt()?;
    let generated_passphrases = root_password
        .generator()
        .length(20)
        .numbers(true)
        .lowercase_letters(true)
        .uppercase_letters(true)
        .symbols(true)
        .spaces(false)
        .exclude_similar_characters(true)
        .strict(true)
        .generate(20)?;

    let selected_passphrase = generated_passphrases
        .passphrases()
        .choose(&mut rand::thread_rng())
        .context("Could not choose generated passphrase")?
        .0;

    generated_passphrases.select(selected_passphrase)?;

    Ok(())
}
