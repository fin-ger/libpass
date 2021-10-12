use std::env;
use std::io::Write;
use std::panic::AssertUnwindSafe;

use cucumber::{given, then, when};
use gpgme::PassphraseRequest;
use pass::{Location, PassphraseProvider, SigningKey, Sorting, StoreBuilder, Umask};

use crate::world::IncrementalWorld;

#[given("a passphrase provider is available")]
fn a_password_provider_is_available(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world
    {
        builder.passphrase_provider(|_req: PassphraseRequest, w: &mut dyn Write| {
            w.write_all(b"test1234\n").unwrap();
            Ok(())
        });
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the system agent is used to unlock passwords")]
fn the_system_agent_is_used_to_unlock_passwords(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world
    {
        builder.passphrase_provider(PassphraseProvider::SystemAgent);
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the password store umask is automatically detected")]
fn the_password_store_umask_is_automatically_detected(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world
    {
        builder.umask(Umask::Automatic);
    }
}

#[given("the password store umask is manually set to 027")]
fn the_password_store_umask_is_manually_set_to_027(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world
    {
        builder.umask(0o027 as u32);
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("a signing key is manually specified")]
fn a_signing_key_is_manually_specified(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world
    {
        builder.signing_key("test@key.email");
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("automatic signing key detection is used")]
fn automatic_signing_key_detection_is_used(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world
    {
        builder.signing_key(SigningKey::Automatic);
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the passwords in the store are sorted")]
fn the_passwords_in_the_store_are_sorted(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world
    {
        builder.sorting(Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST);
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("no signing key is specified")]
fn no_signing_key_is_specified(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { .. } = world {
        // nop
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("a signing key is specified in the environment")]
fn a_signing_key_is_specified_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_SIGNING_KEY", "test@key.email");
        envs.insert(
            "PASSWORD_STORE_SIGNING_KEY".to_owned(),
            "test@key.email".to_owned(),
        );
    }
}

#[given("the password store umask environment variable is set to 027")]
fn the_password_store_umask_environment_variable_is_set_to_027(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_UMASK", "027");
        envs.insert("PASSWORD_STORE_UMASK".to_string(), "027".to_string());
    }
}

#[given("the password store location is set in the environment")]
fn a_password_store_directory_is_set_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, home, .. } = world {
        let path = home.path().join("custom-password-store");
        env::set_var("PASSWORD_STORE_DIR".to_owned(), &path);
        envs.insert(
            "PASSWORD_STORE_DIR".to_owned(),
            format!("{}", path.display()),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("a password store key is set in the environment")]
fn a_password_store_key_is_set_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        let alternative_key = "tobedone"; // TODO
        env::set_var("PASSWORD_STORE_KEY", &alternative_key);
        envs.insert("PASSWORD_STORE_KEY".to_owned(), alternative_key.to_owned());
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the generated password length is set in the environment")]
fn the_generated_password_length_is_set_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_GENERATED_LENGTH", "32");
        envs.insert(
            "PASSWORD_STORE_GENERATED_LENGTH".to_owned(),
            "32".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the character set is set in the environment")]
fn the_character_set_is_set_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_CHARACTER_SET", "[=a=]");
        envs.insert("PASSWORD_STORE_CHARACTER_SET".to_owned(), "a".to_owned());
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the ignored symbols are set in the environment")]
fn the_ignored_symbols_are_set_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS", "[:print:]");
        envs.insert(
            "PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS".to_owned(),
            "[:print:]".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("GPG options are set in the environment")]
fn gpg_options_are_set_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_GPG_OPTS", "--default-recipient intruder");
        envs.insert(
            "PASSWORD_STORE_GPG_OPTS".to_owned(),
            "--default-recipient intruder".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("pass extensions are enabled in the environment")]
fn pass_extensions_are_enabled_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_ENABLE_EXTENSIONS", "1");
        envs.insert(
            "PASSWORD_STORE_ENABLE_EXTENSIONS".to_owned(),
            "1".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("pass extensions directory is set in the environment")]
fn pass_extensions_directory_is_set_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_EXTENSIONS_DIR", "/evil/extensions/dir");
        envs.insert(
            "PASSWORD_STORE_EXTENSIONS_DIR".to_owned(),
            "/evil/extensions/dir".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("a foreign GPG ID is set via the environment")]
fn a_foreign_gpg_id_is_set_via_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_KEY", "test2@key.email");
        envs.insert(
            "PASSWORD_STORE_KEY".to_owned(),
            "test2@key.email".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the X selection is set to clipboard in the environment")]
fn the_x_selection_is_set_to_clipboard_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_X_SELECTION", "clipboard");
        envs.insert(
            "PASSWORD_STORE_X_SELECTION".to_owned(),
            "clipboard".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the X selection is set to primary in the environment")]
fn the_x_selection_is_set_to_primary_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_X_SELECTION", "primary");
        envs.insert(
            "PASSWORD_STORE_X_SELECTION".to_owned(),
            "primary".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the X selection is set to secondary in the environment")]
fn the_x_selection_is_set_to_secondary_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("env").unwrap();
    }

    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_X_SELECTION", "secondary");
        envs.insert(
            "PASSWORD_STORE_X_SELECTION".to_owned(),
            "secondary".to_owned(),
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given(regex = "a new password store is initialized(.*)")]
fn a_new_password_store_is_initialized(world: &mut IncrementalWorld, location: String) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Prepared { home, key_id, .. } = prev {
        let location = match location.as_str() {
            "" => Location::Automatic,
            " at a manually provided location" => {
                let path = home.path().join("custom-password-store");
                Location::Manual(path)
            }
            _ => {
                panic!(
                    "Invalid location '{}' for password store initialization!",
                    location,
                );
            }
        };
        *world = IncrementalWorld::Created {
            store: AssertUnwindSafe(StoreBuilder::default().location(location).init(&key_id)),
            home,
        };
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given(regex = "a password store is opened(.*)")]
fn a_password_store_is_opened(world: &mut IncrementalWorld, location: String) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Prepared { home, .. } = prev {
        let location = match location.as_str() {
            "" => Location::Automatic,
            " at a manually provided location" => {
                let path = home.path().join("custom-password-store");
                Location::Manual(path)
            }
            _ => {
                panic!(
                    "Invalid location '{}' for password store opening!",
                    location,
                );
            }
        };

        *world = IncrementalWorld::Created {
            store: AssertUnwindSafe(StoreBuilder::default().location(location).open()),
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
            panic!(
                "Store had errors when it should not have any: {:?}",
                store.errors().collect::<Vec<_>>(),
            );
        }
    } else {
        panic!("World state is not Successful!");
    }
}
