use std::collections::HashMap;
use std::env;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

use anyhow::Context as AnyhowContext;
use cucumber::World;
use pass::{ConflictResolver, DecryptedPassword, Directory, Password, Store, StoreBuilder, StoreError};
use tempfile::TempDir;

use ouroboros::self_referencing;

#[self_referencing(pub_extras)]
#[derive(Debug)]
pub struct ResolvingStore {
    pub store: Store,
    #[borrows(mut store)]
    #[not_covariant]
    pub resolver: Option<ConflictResolver<'this>>,
}

#[derive(Debug, World)]
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
        envs: HashMap<String, String>,
    },
    Successful {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
    },
    Failure {
        home: TempDir,
    },
    DecryptedPassword {
        password: DecryptedPassword,
    },
    Search {
        found_entries: Vec<PathBuf>,
    },
    Grep {
        found_passwords: Vec<DecryptedPassword>,
    },
    NewPassword {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        password: Password,
        envs: HashMap<String, String>,
    },
    EditedPassword {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        password: Password,
        envs: HashMap<String, String>,
    },
    RemovedPassword {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        path: PathBuf,
        envs: HashMap<String, String>,
    },
    RenamedPassword {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        password: Password,
        envs: HashMap<String, String>,
    },
    NewPasswordAndDirectory {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        password: Password,
        envs: HashMap<String, String>,
    },
    RenamedDirectory {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        directory: Directory,
        envs: HashMap<String, String>,
    },
    RemovedDirectory {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        path: PathBuf,
        envs: HashMap<String, String>,
    },
    Pushed {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
        result: Result<(), git2::Error>,
    },
    Pulled {
        home: TempDir,
        resolving_store: AssertUnwindSafe<ResolvingStore>,
        envs: HashMap<String, String>,
    },
    ConflictAutomaticallyResolved {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
    },
    ConflictManuallyResolved {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
    },
    BinaryConflictManuallyResolved {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
    },
    TextConflictManuallyResolved {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
    },
    GpgIdConflictManuallyResolved {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
    },
    LevelOrderTraversal {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
        entries: Vec<PathBuf>,
    },
    PreOrderTraversal {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
        entries: Vec<PathBuf>,
    },
    PostOrderTraversal {
        home: TempDir,
        store: AssertUnwindSafe<Store>,
        envs: HashMap<String, String>,
        entries: Vec<PathBuf>,
    },
}

impl Default for IncrementalWorld {
    fn default() -> Self {
        Self::Initial
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

        let home = tempfile::Builder::new()
            .prefix(&format!("libpass-{}_", name))
            .tempdir()
            .context(format!(
                "Could not create temporary home folder for {}",
                name
            ))?;
        env::set_var("HOME", home.path());
        env::set_var(
            "GNUPGHOME",
            env::temp_dir().join("libpass-pgp-home").join(".gnupg"),
        );
        let mut envs = HashMap::new();
        envs.insert("HOME".to_string(), home.path().display().to_string());
        envs.insert(
            "GNUPGHOME".to_string(),
            env::temp_dir()
                .join("libpass-pgp-home")
                .join(".gnupg")
                .display()
                .to_string(),
        );

        unsafe {
            // update cached environment variables in libgit2
            libgit2_sys::git_libgit2_shutdown();
            libgit2_sys::git_libgit2_init();
        }

        let key_id = String::from("test@key.email");
        let builder = AssertUnwindSafe(StoreBuilder::default());

        println!(
            "\nClean test environment prepared for {}",
            home.path().display()
        );

        Ok(IncrementalWorld::Prepared {
            envs,
            builder,
            home,
            key_id,
            name,
        })
    }
}
