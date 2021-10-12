use std::panic::AssertUnwindSafe;

use cucumber_rust::{then, when};
use pass::TraversalOrder;

use crate::world::IncrementalWorld;
use crate::{DIR, PW};

#[then("the password store is empty")]
fn the_password_store_is_empty(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let length = store.show(".", TraversalOrder::LevelOrder).unwrap().count();
        if length > 0 {
            panic!(
                "Store is not empty when it should be! Actual length: {}",
                length
            );
        }
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the password store's directory exists")]
fn the_password_stores_directory_exists(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        if !store.location().exists() {
            panic!(
                "Store directory does not exist when it should exist! Path: {}",
                store.location().display(),
            );
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
            panic!(
                "Store directory exists when it shouldn't! Path: {}",
                path.display(),
            );
        }
    } else {
        panic!("World state is not Failure!");
    }
}

#[then("the password store's directory contains a GPG ID file")]
fn the_password_stores_directory_contains_a_gpg_id_file(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let path = store.location().join(".gpg-id");
        if !path.exists() {
            panic!(
                "Store directory does not contain a GPG ID file! Path: {}",
                path.display(),
            );
        }
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the password store contains passwords")]
fn the_password_store_contains_passwords(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, home } =
        std::mem::replace(world, IncrementalWorld::Initial)
    {
        let expected = [
            (DIR, ("", ".")),
            (DIR, ("Entertainment", "Entertainment")),
            (DIR, ("Manufacturers", "Manufacturers")),
            (PW, ("Phone.gpg", "Phone")),
            (DIR, ("Entertainment/Holo Deck", "Holo Deck")),
            (PW, ("Manufacturers/Sokor.gpg", "Sokor")),
            (PW, ("Manufacturers/StrutCo.gpg", "StrutCo")),
            (PW, ("Manufacturers/Yoyodyne.gpg", "Yoyodyne")),
            (PW, ("Entertainment/Holo Deck/Broht & Forrester.gpg", "Broht & Forrester")),
        ];
        let store = store.0;
        let actual = store.show(".", TraversalOrder::LevelOrder).unwrap();

        for (ref entry, (is_dir, (expected_path, expected_name))) in actual.zip(&expected) {
            if *is_dir {
                assert!(
                    entry.is_dir(),
                    "{} is not a directory",
                    entry.path().display(),
                );
            } else {
                assert!(
                    entry.is_password(),
                    "{} is not a password",
                    entry.path().display(),
                );
            }
            assert!(
                entry.path().ends_with(expected_path),
                "path {} has no suffix {}",
                entry.path().display(), expected_path,
            );
            assert!(
                entry.name() == *expected_name,
                "name {} is not {}",
                entry.name(), expected_name,
            );
        }

        *world = IncrementalWorld::Successful {
            store: AssertUnwindSafe(store),
            home,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a password is opened")]
fn a_password_is_opened(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let entry = store.show("./Manufacturers/StrutCo.gpg", TraversalOrder::PreOrder).unwrap()
            .next().expect("Manufacturers/StrutCo password not found in password store!");

        let strutco = entry.password()
            .expect("Manufacturers/StrutCo is not a password but a directory!");
        let password = strutco
            .decrypt()
            .expect("Decrypting Manufacturers/StrutCo failed!");
        *world = IncrementalWorld::DecryptedPassword { password };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("an existing password is searched in the password store")]
fn an_existing_password_is_searched_in_the_password_store(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let found_entries = store
            .find("strutco")
            .map(|entry| entry.path().to_owned())
            .collect::<Vec<_>>();
        *world = IncrementalWorld::Search { found_entries };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a non-existent password is searched in the password store")]
fn a_non_existent_password_is_searched_in_the_password_store(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let found_entries = store
            .find("romulan warbird access codes")
            .map(|entry| entry.path().to_owned())
            .collect::<Vec<_>>();
        *world = IncrementalWorld::Search { found_entries };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("content of an existing password is searched in the password store")]
fn content_of_an_existing_password_is_searched_in_the_password_store(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let found_passwords = store
            .grep("pattern")
            .map(|password| password.decrypt().expect("could not decrypt password"))
            .collect::<Vec<_>>();
        *world = IncrementalWorld::Grep { found_passwords };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("content of a non-existing password is searched in the password store")]
fn content_of_a_non_existing_password_is_searched_in_the_password_store(
    world: &mut IncrementalWorld,
) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let found_passwords = store
            .grep("romulan cloaking frequency")
            .map(|password| password.decrypt().expect("could not decrypt password"))
            .collect::<Vec<_>>();
        *world = IncrementalWorld::Grep { found_passwords };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a new password is created")]
fn a_new_password_is_created(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let mut root = store.show("./", TraversalOrder::LevelOrder)
            .expect("could not get root directory of password store")
            .next()
            .expect("could not get root directory of password store")
            .directory()
            .expect("Root directory is not a directory");

        let password = root.password_insertion("Shuttle Bay")
            .passphrase("0p3n-5354m3")
            .insert(store)
            .expect("Password insertion failed");

        *world = IncrementalWorld::NewPassword { password };
    } else {
        panic!("World state is not Successful!");
    }
}
