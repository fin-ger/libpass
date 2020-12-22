use std::panic::AssertUnwindSafe;

use cucumber_rust::{given, when, then};
use pass::{Store, Location};

use crate::world::IncrementalWorld;

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
            store: AssertUnwindSafe(Store::init(location, key_id)),
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
