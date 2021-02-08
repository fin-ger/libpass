use std::panic::AssertUnwindSafe;

use cucumber_rust::{when, then};
use pass::{Sorting, TraversalOrder};

use crate::world::IncrementalWorld;
use crate::{DIR, PW};

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

#[when("a password is opened")]
fn a_password_is_opened(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let content = store.content();
        let filter = content.directories().filter(|d| d.name() == "Manufacturers").next();
        let manufacturers = if let Some(dir) = filter { dir } else {
            panic!("Manufacturers directory not found in password store!");
        };

        let filter = manufacturers.passwords().filter(|pw| pw.name() == "StrutCo").next();
        let strutco = if let Some(pw) = filter { pw } else {
            panic!("Manufacturers/StrutCo password not found in password store!");
        };

        let password = strutco.decrypt().expect("Decrypting Manufacturers/StrutCo failed");
        *world = IncrementalWorld::DecryptedPassword { password };
    } else {
        panic!("World state is not Successful!");
    }
}
