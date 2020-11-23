use std::convert::Infallible;
use std::env;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::panic::AssertUnwindSafe;

use anyhow::{bail, Context as AnyhowContext};
use tempdir::TempDir;
use gpgme::{Context, Protocol, PassphraseRequest, PinentryMode};
use async_trait::async_trait;
use cucumber_rust::{given, then, when, World, WorldInit};
use pass::{Store, StoreError, Location, Sorting, TraversalOrder};

//it should create a new password store if none available with provided GPG keys
//it should create a new password store if none available when git is not configured
//it should create a new password store if none available when there is no GPG ID
//it should open an existing password store at the default location
//it should open an existing password store from the environment variable
//it should open an existing password store at a manually provided location
//it should be able to set GPG id's for subdirectories in the password store
//it should be able to traverse in level-order over all entries in the store
//it should be able to traverse in pre-order over all entries in the store
//it should be able to traverse in post-order over all entries in the store
//it should be able to programmatically walk over all directories contained in the store
//it sbould be able to programmatically walk over all passwords contained in the store
//it should use the next-in-parents gpg-id's stored in the password store to decrypt passwords
//it should use the next-in-parents gpg-id's stored in the password store to encrypt passwords
//it should be able to create a QR code for each field in the decrypted password and for the whole file
//it should be possible to provide a custom password provider for the whole password store (instead of the system agent)
//it should use the gpg-id's from the environment variable if specified to decrypt passwords
//it should use the gpg-id's from the environment variable if specified to encrypt passwords
//it should be able to create a git commit when a new password was created
//it should be able to create a git commit when a password was edited
//it should be able to create a git commit when a password was removed
//it should be able to create a git commit when a password was copied
//it should be able to create a git commit when a new directory was created
//it should be able to create a git commit when a directory was renamed
//it should be able to create a git commit when a directory was removed
//it should be able to query the status of the git repository
//it should be able to push to the git remote if fast-forward
//it should be able to pull fast-forward changes from the remote without interaction
//it should be able to pull non-fast-forward changes from the remote with automatic merging
//it should be able to resolve a merge-conflict while pulling by letting the user resolve the conflict from decrpyted passwords
//it should be able to search for a filename in the password store
//it should be able to search for password-content in the password store
//it should be able to generate a password with respect to symbols and length
//it should be able to generate a password with respect to PASSWORD_STORE_GENERATED_LENGTH
//it should be able to generate a password with respect to PASSWORD_STORE_CHARACTER_SET
//it should be able to generate a password with respect to PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS
//it should sign gpg-id files according to PASSWORD_STORE_SIGNING_KEY's
//it should place a password in the clipboard for PASSWORD_STORE_CLIP_TIME seconds
//it should place a password in the clipboard with respect to PASSWORD_STORE_X_SELECTION
//it should modify all files in the password store with respect to PASSWORD_STORE_UMASK
//it should warn the user if PASSWORD_STORE_ENABLE_EXTENSIONS is set (no support)
//it should warn the user if PASSWORD_STORE_EXTENSIONS_DIR is set (no support)
//it should (for now) warn the user if PASSWORD_STORE_GPG_OPTS is set as these cannot be parsed by the library

const DIR: bool = true;
const PW: bool = false;

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

fn clean_env(name: &'static str) -> anyhow::Result<IncrementalWorld> {
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

    let key_id = create_pgp_id().unwrap();

    Ok(IncrementalWorld::Prepared {
        envs,
        home,
        key_id,
        name,
    })
}

fn create_pgp_id() -> anyhow::Result<String> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)
        .context("Could not create GPG context")?
        .set_passphrase_provider(|_req: PassphraseRequest, out: &mut dyn Write| {
            out.write_all(b"test1234").unwrap();
            Ok(())
        });
    ctx.set_pinentry_mode(PinentryMode::Loopback)
        .context("Could not set pinentry mode in GPG")?;

    let result = ctx.create_key("Test Key <test@key.email>", "default", None)
        .context("Failed to create GPG key")?;
    let key_id = match result.fingerprint() {
        Ok(key_id) => key_id,
        Err(_err) => bail!("Could not get fingerprint of new GPG key"),
    };

    Ok(key_id.to_string())
}


#[given("no password store exists")]
fn no_password_store_exists(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = clean_env("no-password-store").unwrap();
    } else {
        panic!("World state is not Initial!");
    }
}

#[given(regex = "a password store exists (.*)")]
fn a_password_store_exists(world: &mut IncrementalWorld, location: String) {
    if let IncrementalWorld::Initial = world {
        *world = clean_env("password-store").unwrap();
        if let IncrementalWorld::Prepared { envs, key_id, home, .. } = world {
            match location.as_str() {
                "at the default location" | "from the environment variable" => {},
                "at a manually provided location" => {
                    let path = home.path().join("custom-password-store");
                    env::set_var("PASSWORD_STORE_DIR", &path);
                    envs.insert("PASSWORD_STORE_DIR".to_owned(), format!("{}", path.display()));
                },
                _ => {
                    panic!(format!(
                        "Invalid location '{}' for password insertion!",
                        location,
                    ));
                },

            };
            let status = Command::new("pass")
                .args(&["init", key_id.as_str()])
                .envs(envs)
                .status()
                .unwrap();
            assert!(status.success(), "Failed to initialize pass repository!");
        } else {
            panic!("World state is not Prepared!");
        }
    } else {
        panic!("World state is not Initial!");
    }
}

fn insert_password(envs: &HashMap<String, String>, name: &str, content: &str) {
    let mut child = Command::new("pass")
        .args(&["insert", "--multiline", name])
        .envs(envs)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(content.as_bytes()).unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "Failed to insert password into pass repository!");
}

#[given(regex = "passwords are stored in the password store")]
fn passwords_are_stored_in_the_password_store(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, .. } = world {
        insert_password(
            envs,
            "Manufacturers/Yoyodyne",
            "all1the%fancy@panels+are;for<me\n\nuser: laforge\n",
        );
        insert_password(
            envs,
            "Phone",
            "PIN: 1701\n\nPattern:\nO--O--5\n|  |  |\nO--4--3\n|  |  |\nO--1--2\n",
        );
        insert_password(
            envs,
            "Manufacturers/StrutCo",
            "i*aint*got*no*tape",
        );
        insert_password(
            envs,
            "Manufacturers/Sokor",
            "pum-yIghoSQo'\nBetter not tell Picard about this.\n",
        );
        insert_password(
            envs,
            "Entertainment/Holo Deck/Broht & Forrester",
            "fun-times1337\nusername: geordi\n",
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[when("a new password store is initialized")]
fn a_new_password_store_is_initialized(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Prepared { home, key_id, .. } = prev {
        *world = IncrementalWorld::Created {
            store: AssertUnwindSafe(Store::init(Location::Automatic, key_id)),
            home,
        };
    } else {
        panic!("World state is not Prepared!");
    }
}

#[when(regex = "a password store is opened (.*)")]
fn a_password_store_is_opened(world: &mut IncrementalWorld, location: String) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Prepared { home, .. } = prev {
        let location = match location.as_str() {
            "at the default location" | "from the environment variable" => {
                Location::Automatic
            },
            "at a manually provided location" => {
                let path = home.path().join("custom-password-store");
                Location::Manual(path)
            },
            _ => {
                panic!(format!(
                    "Invalid location '{}' for password store opening!",
                    location,
                ));
            },
        };
        *world = IncrementalWorld::Created {
            store: AssertUnwindSafe(Store::open(location)),
            home,
        };
    } else {
        panic!("World state is not Prepared!");
    }
}

#[when(regex = "(.*) password store is successfully (.*)")]
fn password_store_is_successfully(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Created { store, home } = prev {
        *world = IncrementalWorld::Successful {
            store: AssertUnwindSafe(store.0.unwrap()),
            home,
        };
    } else {
        panic!("World state is not Created!");
    }
}

#[then("the initialization of the password store fails")]
fn the_initialization_of_the_password_store_fails(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Created { store, home } = prev {
        if store.0.is_err() {
            *world = IncrementalWorld::Failure { home };
        } else {
            panic!("Store creation did not fail when it should have!");
        }
    } else {
        panic!("World state is not Created!");
    }
}

#[then("the password store has no errors")]
fn the_password_store_has_no_errors(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        if store.has_errors() {
            panic!(format!(
                "Store had errors when it should not have any: {:?}",
                store.errors().collect::<Vec<_>>(),
            ));
        }
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the password store is empty")]
fn the_password_store_is_empty(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let length = store.traverse_recursive(TraversalOrder::LevelOrder).count();
        if length > 0 {
            panic!(format!(
                "Store is not empty when it should be! Actual length: {}",
                length
            ));
        }
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the password store's directory exists")]
fn the_password_stores_directory_exists(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        if !store.location().exists() {
            panic!(format!(
                "Store directory does not exist when it should exist! Path: {}",
                store.location().display(),
            ));
        }
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the password store's directory contains a GPG ID file")]
fn the_password_stores_directory_contains_a_gpg_id_file(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let path = store.location().join(".gpg-id");
        if !path.exists() {
            panic!(format!(
                "Store directory does not contain a GPG ID file! Path: {}",
                path.display(),
            ));
        }
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the password store's directory does not exist")]
fn the_password_stores_directory_does_not_exist(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Failure { home } = world {
        let path = home.path().join(".password-store");
        if path.exists() {
            panic!(format!(
                "Store directory exists when it shouldn't! Path: {}",
                path.display(),
            ));
        }
    } else {
        panic!("World state is not Failure!");
    }
}

#[then("the password store contains passwords")]
fn the_password_store_contains_passwords(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, home } = std::mem::replace(world, IncrementalWorld::Initial) {
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
        let store = store.0.with_sorting(Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST);
        let actual = store.traverse_recursive(TraversalOrder::LevelOrder);

        for (ref entry, (is_dir, expected)) in actual.zip(&expected) {
            if *is_dir {
                assert!(
                    entry.is_dir(),
                    format!("{} is not a directory", entry.path().display()),
                );
            } else {
                assert!(
                    entry.is_password(),
                    format!("{} is not a password", entry.path().display()),
                );
            }
            assert!(
                entry.path().ends_with(expected),
                format!("{} is not {}", entry.path().display(), expected),
            );
        }

        *world = IncrementalWorld::Successful { store: AssertUnwindSafe(store), home };
    } else {
        panic!("World state is not Successful!");
    }
}

fn main() {
    let runner = IncrementalWorld::init(&["./features"]);

    // You may choose any executor you like (Tokio, async-std, etc)
    // You may even have an async main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    futures::executor::block_on(runner.run());
}
