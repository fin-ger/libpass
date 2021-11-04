use std::panic::AssertUnwindSafe;
use std::process::{Command, Stdio};

use cucumber::{then, when};
use pass::GitRemote;
use pass::{Traversal, TraversalOrder};

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
    if let IncrementalWorld::Successful { store, home, envs } =
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
            envs,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the repository is clean and contains a new commit")]
fn the_repository_is_clean_and_contains_a_new_commit(world: &mut IncrementalWorld) {
    let envs = match world {
        IncrementalWorld::NewPassword { envs, .. } => envs,
        IncrementalWorld::EditedPassword { envs, .. } => envs,
        IncrementalWorld::RemovedPassword { envs, .. } => envs,
        IncrementalWorld::RenamedPassword { envs, .. } => envs,
        IncrementalWorld::NewPasswordAndDirectory { envs, .. } => envs,
        IncrementalWorld::RenamedDirectory { envs, .. } => envs,
        IncrementalWorld::RemovedDirectory { envs, .. } => envs,
        _ => panic!("World state is invalid!"),
    };

    let output = Command::new("pass")
        .args(&["git", "status", "--porcelain"])
        .envs(envs.clone())
        .stdout(Stdio::piped())
        .output()
        .expect("Could not check git state");
    let stdout = String::from_utf8(output.stdout)
        .expect("Could not read stdout as UTF-8");
    assert_eq!(stdout, "", "Repository not clean");

    let output = Command::new("pass")
        .args(&["git", "log", "--pretty=format:%s"])
        .envs(envs.clone())
        .stdout(Stdio::piped())
        .output()
        .expect("Could not check git commit");
    let stdout = String::from_utf8(output.stdout)
        .expect("Could not read stdout as UTF-8");

    match world {
        IncrementalWorld::NewPassword { envs, .. } => {
            assert_eq!(stdout.lines().count(), 6, "Not enough commits");
            assert_eq!(stdout.lines().next().unwrap(), "Add password for 'Ready Room' using libpass.");

            let output = Command::new("pass")
                .args(&["show", "Ready Room"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not read Ready Room password content");
            let pw_content = String::from_utf8(output.stdout)
                .expect("Cloud not read stdout as UTF-8");

            assert_eq!(pw_content, "what-are-our-options\n");
        }
        IncrementalWorld::EditedPassword { envs, .. } => {
            assert_eq!(stdout.lines().count(), 6, "Not enough commits");
            assert_eq!(stdout.lines().next().unwrap(), "Edit password for 'Manufacturers/Sokor' using libpass.");

            let output = Command::new("pass")
                .args(&["show", "Manufacturers/Sokor"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not read Sokor password content");
            let pw_content = String::from_utf8(output.stdout)
                .expect("Cloud not read stdout as UTF-8");

            assert_eq!(pw_content, "pum-yIghoSQo'\nBetter not tell Picard about this.\nNote: Picard already knows...\n");
        }
        IncrementalWorld::RemovedPassword { envs, .. } => {
            assert_eq!(stdout.lines().count(), 6, "Not enough commits");
            assert_eq!(stdout.lines().next().unwrap(), "Remove 'Manufacturers/Sokor' from store.");

            let status = Command::new("pass")
                .args(&["show", "Manufacturers/Sokor"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .status()
                .expect("Could not read Sokor password content");

            assert!(!status.success(), "Password Sokor has not been removed!");
        }
        IncrementalWorld::RenamedPassword { envs, .. } => {
            assert_eq!(stdout.lines().count(), 6, "Not enough commits");
            assert_eq!(stdout.lines().next().unwrap(), "Rename 'Manufacturers/Sokor' to 'Manufacturers/None of your concern'.");

            let status = Command::new("pass")
                .args(&["show", "Manufacturers/None of your concern"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .status()
                .expect("Could not read Sokor password content");

            assert!(status.success(), "Sokor password has not been renamed!");
        }
        IncrementalWorld::NewPasswordAndDirectory { envs, .. } => {
            assert_eq!(stdout.lines().count(), 1, "Not enough commits");
            assert_eq!(stdout.lines().next().unwrap(), "Add password for 'Warp Nacelles/Starfleet' using libpass.");

            let output = Command::new("pass")
                .args(&["show", "Warp Nacelles/Starfleet"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not read Warp Nacelles/Starfleet password content");
            let pw_content = String::from_utf8(output.stdout)
                .expect("Cloud not read stdout as UTF-8");

            assert_eq!(pw_content, "two-nacelles-ftw\n");
        }
        IncrementalWorld::RenamedDirectory { envs, .. } => {
            assert_eq!(stdout.lines().count(), 6, "Not enough commits");
            assert_eq!(stdout.lines().next().unwrap(), "Rename 'Entertainment/Holo Deck' to 'Entertainment/Novels'.");

            let status = Command::new("pass")
                .args(&["show", "Entertainment/Novels"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .status()
                .expect("Could not read Novels directory content");

            assert!(status.success(), "Novels directory has not been renamed!");
        }
        IncrementalWorld::RemovedDirectory { envs, .. } => {
            assert_eq!(stdout.lines().count(), 6, "Not enough commits");
            assert_eq!(stdout.lines().next().unwrap(), "Remove 'Entertainment' from store.");

            let status = Command::new("pass")
                .args(&["show", "Entertainment"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .status()
                .expect("Could not read Entertainment directory content");

            assert!(!status.success(), "Entertainment directory has not been removed!");
        }
        _ => unreachable!(),
    };
}

#[then("the repository is clean")]
fn the_repository_is_clean(world: &mut IncrementalWorld) {
    let envs = match world {
        IncrementalWorld::Successful { envs, .. } => envs,
        _ => panic!("World state is invalid!"),
    };

    let output = Command::new("pass")
        .args(&["git", "status", "--porcelain"])
        .envs(envs.clone())
        .stdout(Stdio::piped())
        .output()
        .expect("Could not check git state");
    let stdout = String::from_utf8(output.stdout)
        .expect("Could not read stdout as UTF-8");
    assert_eq!(stdout, "", "Repository is not clean!");
}

#[then("pushing the commit succeeds")]
fn pushing_the_commit_succeeds(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pushed { result, envs, .. } = prev {
        result.expect("Failed to push to remote");

        let output = Command::new("pass")
            .args(&["git", "log", "origin/master..master"])
            .envs(envs.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Could not check git state");
        let stdout = String::from_utf8(output.stdout)
            .expect("Could not read stdout as UTF-8");
        let stderr = String::from_utf8(output.stderr)
            .expect("Could not read stderr as UTF-8");
        assert_eq!(stderr, "", "Errors occurred while checking git for pushable commits!");
        assert_eq!(stdout, "", "Commits are not pushed!");
    } else {
        panic!("World state is not Pushed!");
    }
}

#[then("pushing the commit fails")]
fn pushing_the_commit_fails(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pushed { result, envs, home, .. } = prev {
        assert!(result.is_err(), "Pushing has not failed, although it should have!");

        let output = Command::new("pass")
            .args(&["git", "log", "origin/master..master"])
            .envs(envs.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Could not check git state");
        let stdout = String::from_utf8(output.stdout)
            .expect("Could not read stdout as UTF-8");
        let stderr = String::from_utf8(output.stderr)
            .expect("Could not read stderr as UTF-8");
        assert_eq!(stderr, "", "Errors occurred while checking git for pushable commits!");
        assert!(!stdout.is_empty(), "Commits are not pushed!");
    } else {
        panic!("World state is not Pushed!");
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

#[when(regex = r"^a (\w+ )?password is created$")]
fn a_new_password_is_created(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful { mut store, home, envs } = prev {
        let root = store.show("./", TraversalOrder::LevelOrder)
            .expect("could not get root directory of password store")
            .next()
            .expect("could not get root directory of password store")
            .directory()
            .expect("Root directory is not a directory");

        let password = root.password_insertion("Ready Room")
            .passphrase("what-are-our-options")
            .insert(&mut store)
            .expect("Password insertion failed");

        *world = IncrementalWorld::NewPassword { store, home, envs, password };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a password is edited")]
fn a_password_is_edited(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful { mut store, home, envs } = prev {
        let password = store.show("Manufacturers/Sokor", TraversalOrder::LevelOrder)
            .expect("could not find Sokor password")
            .next()
            .expect("could not find Sokor password")
            .password()
            .expect("Sokor is not a password");
        password
            .decrypt()
            .expect("Could not decrypt Sokor")
            .append_line(&mut store, "Note: Picard already knows...")
            .expect("Failed to append line to Sokor");

        *world = IncrementalWorld::EditedPassword { store, home, envs, password };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a password is removed")]
fn a_password_is_removed(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful { mut store, home, envs } = prev {
        let password = store.show("Manufacturers/Sokor", TraversalOrder::LevelOrder)
            .expect("could not find Sokor password")
            .next()
            .expect("could not find Sokor password")
            .password()
            .expect("Sokor is not a password");
        let path = password.path().to_owned();
        password
            .make_mut(&mut store)
            .remove()
            .expect("Could not remove password");

        *world = IncrementalWorld::RemovedPassword { store, home, envs, path };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a password is renamed")]
fn a_password_is_renamed(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful { mut store, home, envs } = prev {
        let mut password = store.show("Manufacturers/Sokor", TraversalOrder::LevelOrder)
            .expect("could not find Sokor password")
            .next()
            .expect("could not find Sokor password")
            .password()
            .expect("Sokor is not a password")
            .make_mut(&mut store);
        password
            .rename("None of your concern")
            .expect("Could not rename password");
        let password = password.make_immut();

        *world = IncrementalWorld::RenamedPassword { store, home, envs, password };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a directory is created")]
fn a_directory_is_created(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { ref mut store, .. } = world {
        let root = store.show(".", TraversalOrder::LevelOrder)
            .expect("could not find root directory")
            .next()
            .expect("could not find root directory")
            .directory()
            .expect("Root is not a directory");

        root
            .directory_insertion("Warp Nacelles")
            .insert(store)
            .expect("Could not create Wrap Nacelles directory");
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a password is created in the new directory")]
fn a_password_is_created_in_the_new_directory(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful { mut store, home, envs } = prev {
        let warp_nacelles = store.show("./Warp Nacelles", TraversalOrder::LevelOrder)
            .expect("could not get root directory of password store")
            .next()
            .expect("could not get root directory of password store")
            .directory()
            .expect("Root directory is not a directory");

        let password = warp_nacelles.password_insertion("Starfleet")
            .passphrase("two-nacelles-ftw")
            .insert(&mut store)
            .expect("Password insertion failed");

        *world = IncrementalWorld::NewPasswordAndDirectory { store, home, envs, password };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a directory is renamed")]
fn a_directory_is_renamed(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful { mut store, home, envs } = prev {
        let mut holo_deck = store.show("Entertainment/Holo Deck", TraversalOrder::LevelOrder)
            .expect("could not find Entertainment/Holo Deck directory")
            .next()
            .expect("could not find Entertainment/Holo Deck directory")
            .directory()
            .expect("Holo Deck is not a directory")
            .make_mut(&mut store);
        holo_deck
            .rename("Novels")
            .expect("Could not rename directory");
        let directory = holo_deck.make_immut();

        *world = IncrementalWorld::RenamedDirectory { store, home, envs, directory };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a directory is removed")]
fn a_directory_is_removed(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful { mut store, home, envs } = prev {
        let entertainment = store.show("Entertainment", TraversalOrder::LevelOrder)
            .expect("could not find Entertainment directory")
            .next()
            .expect("could not find Entertainment directory")
            .directory()
            .expect("Entertainment is not a directory")
            .make_mut(&mut store);
        let path = entertainment.path().to_owned();
        entertainment
            .remove(Traversal::Recursive)
            .expect("Could not remove directory");

        *world = IncrementalWorld::RemovedDirectory { store, home, envs, path };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("the commit is pushed to the remote")]
fn the_commit_is_pushed_to_the_remote(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::NewPassword { mut store, home, envs, .. } = prev {
        let result = store
            .git().expect("Store not using git")
            .push(GitRemote::UpstreamForBranch);

        *world = IncrementalWorld::Pushed { store, home, envs, result };
    } else {
        panic!("World state is not Successful!");
    }
}
