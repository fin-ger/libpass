use std::convert::Infallible;
use std::env;
use std::collections::HashMap;
use std::io::Cursor;
use std::panic::AssertUnwindSafe;

use anyhow::Context as AnyhowContext;
use tempdir::TempDir;
use async_trait::async_trait;
use cucumber_rust::{World, WorldInit};
use pass::{Store, StoreError};
use base64::read::DecoderReader;
use zstd::stream::read::Decoder;
use tar::Archive;

#[derive(WorldInit)]
pub enum IncrementalWorld {
    // You can use this struct for mutable context in scenarios.
    Initial,
    Prepared {
        envs: HashMap<String, String>,
        home: TempDir,
        key_id: String,
        name: &'static str,
    },
    Created {
        home: TempDir,
        store: AssertUnwindSafe<Result<Store, StoreError>>,
    },
    Successful {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
    },
    Failure {
        home: TempDir,
    },
}

#[async_trait(?Send)]
impl World for IncrementalWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self::Initial)
    }
}

impl IncrementalWorld {
    pub fn clean_env(name: &'static str) -> anyhow::Result<IncrementalWorld> {
        env::remove_var("PASSWORD_STORE_DIR");
        env::remove_var("PASSWORD_STORE_KEY");
        env::remove_var("PASSWORD_STORE_GPG_OPTS");
        env::remove_var("PASSWORD_STORE_X_SELECTION");
        env::remove_var("PASSWORD_STORE_CLIP_TIME");
        env::remove_var("PASSWORD_STORE_UMASK");
        env::remove_var("PASSWORD_STORE_GENERATED_LENGTH");
        env::remove_var("PASSWORD_STORE_CHARACTER_SET");
        env::remove_var("PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS");
        env::remove_var("PASSWORD_STORE_ENABLE_EXTENSIONS");
        env::remove_var("PASSWORD_STORE_EXTENSIONS_DIR");
        env::remove_var("PASSWORD_STORE_SIGNING_KEY");

        let home = TempDir::new(&format!("libpass-{}", name))
            .context(format!("Could not create temporary home folder for {}", name))?;
        env::set_var("HOME", home.path());
        let mut envs = HashMap::new();
        envs.insert("HOME".to_string(), home.path().display().to_string());

        let key_id = initialize_pgp_home()?;

        Ok(IncrementalWorld::Prepared {
            envs,
            home,
            key_id,
            name,
        })
    }
}

fn initialize_pgp_home() -> anyhow::Result<String> {
    let key_id = "75B1299A3994E45FA0E9E5CA5737788D11320265";
    let b64_tar = "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";

    let mut cursor = Cursor::new(b64_tar);
    let decoder = DecoderReader::new(&mut cursor, base64::STANDARD);
    let decompressor = Decoder::new(decoder)?;
    let mut archive = Archive::new(decompressor);
    let home = env::var("HOME")?;
    archive.unpack(home)?;
    Ok(key_id.to_string())
}
