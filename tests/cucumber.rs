use std::convert::Infallible;
use std::env;
use std::collections::HashMap;
use std::io::{Write, Cursor};
use std::process::{Command, Stdio};
use std::panic::AssertUnwindSafe;

use anyhow::Context as AnyhowContext;
use tempdir::TempDir;
use async_trait::async_trait;
use cucumber_rust::{given, then, when, World, WorldInit};
use pass::{Store, StoreError, Location, Sorting, TraversalOrder};
use base64::read::DecoderReader;
use zstd::stream::read::Decoder;
use tar::Archive;

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

    let key_id = initialize_pgp_home()?;

    Ok(IncrementalWorld::Prepared {
        envs,
        home,
        key_id,
        name,
    })
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

#[given("a new password store is initialized")]
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

#[given(regex = "a password store is opened (.*)")]
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

#[then(regex = "the (.*) of the password store fails")]
fn the_operation_of_the_password_store_fails(world: &mut IncrementalWorld) {
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
