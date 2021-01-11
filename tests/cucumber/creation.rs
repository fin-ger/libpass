use std::panic::AssertUnwindSafe;
use std::io::Write;
use std::env;

use cucumber_rust::{given, when, then};
use pass::{StoreBuilder, Location, PassphraseProvider, Umask};
use gpgme::PassphraseRequest;

use crate::world::IncrementalWorld;

#[given("a passphrase provider is available")]
fn a_password_provider_is_available(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world {
        *builder = builder.clone().passphrase_provider(
            |_req: PassphraseRequest, w: &mut dyn Write| {
                w.write_all(b"test1234\n").unwrap();
                Ok(())
            }
        );
    }
}

#[given("the system agent is used to unlock passwords")]
fn the_system_agent_is_used_to_unlock_passwords(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world {
        *builder = builder
            .clone()
            .passphrase_provider(PassphraseProvider::SystemAgent);
    }
}

#[given("the password store umask is automatically detected")]
fn the_password_store_umask_is_automatically_detected(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world {
        *builder = builder
            .clone()
            .umask(Umask::Automatic);
    }
}

#[given("the password store umask is manually set to 027")]
fn the_password_store_umask_is_manually_set_to_027(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared {
        builder: AssertUnwindSafe(ref mut builder),
        ..
    } = world {
        *builder = builder
            .clone()
            .umask(0o027 as u32);
    }
}

#[given("the password store umask environment variable is set to 027")]
fn the_password_store_umask_environment_variable_is_set_to_027(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, .. } = world {
        env::set_var("PASSWORD_STORE_UMASK", "027");
        envs.insert("PASSWORD_STORE_UMASK".to_string(), "027".to_string());
    }
}

#[given("a password store directory is set in the environment")]
fn a_password_store_directory_is_set_in_the_environment(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, .. } = world {
        //env::set_var("PASSWORD_STORE_DIR",
        // TODO
    }
}

#[given(regex = "a new password store is initialized(.*)")]
fn a_new_password_store_is_initialized(world: &mut IncrementalWorld, location: String) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Prepared { home, key_id, .. } = prev {
        let location = match location.as_str() {
            "" => {
                Location::Automatic
            },
            " at a manually provided location" => {
                let path = home.path().join("custom-password-store");
                Location::Manual(path)
            },
            _ => {
                panic!(format!(
                    "Invalid location '{}' for password store initialization!",
                    location,
                ));
            },
        };
        *world = IncrementalWorld::Created {
            store: AssertUnwindSafe(
                StoreBuilder::default()
                    .location(location)
                    .init(&key_id)
            ),
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
            "" => {
                Location::Automatic
            },
            " at a manually provided location" => {
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
            store: AssertUnwindSafe(
                StoreBuilder::default()
                    .location(location)
                    .open()
            ),
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
