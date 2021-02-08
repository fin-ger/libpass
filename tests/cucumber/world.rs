use std::convert::Infallible;
use std::env;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;

use anyhow::Context as AnyhowContext;
use tempdir::TempDir;
use async_trait::async_trait;
use cucumber_rust::{World, WorldInit};
use pass::{Store, StoreError, StoreBuilder, DecryptedPassword};

#[derive(WorldInit)]
pub enum IncrementalWorld {
    // You can use this struct for mutable context in scenarios.
    Initial,
    Prepared {
        envs: HashMap<String, String>,
        builder: AssertUnwindSafe<StoreBuilder>,
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
    DecryptedPassword {
        password: DecryptedPassword,
    }
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
        env::set_var("GNUPGHOME", env::temp_dir().join("libpass-pgp-home").join(".gnupg"));
        let mut envs = HashMap::new();
        envs.insert("HOME".to_string(), home.path().display().to_string());
        envs.insert("GNUPGHOME".to_string(), env::temp_dir().join("libpass-pgp-home").join(".gnupg").display().to_string());

        let key_id = String::from("test@key.email");
        let builder = AssertUnwindSafe(StoreBuilder::default());

        Ok(IncrementalWorld::Prepared {
            envs,
            builder,
            home,
            key_id,
            name,
        })
    }
}
