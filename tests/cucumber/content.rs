use std::panic::AssertUnwindSafe;
use std::process::{Command, Stdio};
use std::path::PathBuf;

use cucumber::{then, when};
use pass::{GitRemote, Store, GpgKeyId, BranchStatus};
use pass::{Traversal, TraversalOrder, PasswordChange, EntryKind};

use crate::world::{IncrementalWorld, ResolvingStoreBuilder};
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
            (
                PW,
                (
                    "Entertainment/Holo Deck/Broht & Forrester.gpg",
                    "Broht & Forrester",
                ),
            ),
        ];
        let store = store.0;
        let actual = store.show(".", TraversalOrder::LevelOrder).unwrap();

        for (ref entry, (is_dir, (expected_path, expected_name))) in actual.zip(&expected) {
            if *is_dir {
                assert!(
                    entry.kind() == EntryKind::Directory,
                    "{} is not a directory",
                    entry.path().display(),
                );
            } else {
                assert!(
                    entry.kind() == EntryKind::Password,
                    "{} is not a password",
                    entry.path().display(),
                );
            }
            assert!(
                entry.path().ends_with(expected_path),
                "path {} has no suffix {}",
                entry.path().display(),
                expected_path,
            );
            assert!(
                entry.name() == *expected_name,
                "name {} is not {}",
                entry.name(),
                expected_name,
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
    let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");
    assert_eq!(stdout, "", "Repository not clean");

    let output = Command::new("pass")
        .args(&["git", "log", "--pretty=format:%s"])
        .envs(envs.clone())
        .stdout(Stdio::piped())
        .output()
        .expect("Could not check git commit");
    let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");

    match world {
        IncrementalWorld::NewPassword { envs, .. } => {
            assert_eq!(stdout.lines().count(), 8, "Not enough commits");
            assert_eq!(
                stdout.lines().next().unwrap(),
                "Add password for 'Ready Room' using libpass."
            );

            let output = Command::new("pass")
                .args(&["show", "Ready Room"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not read Ready Room password content");
            let pw_content =
                String::from_utf8(output.stdout).expect("Cloud not read stdout as UTF-8");

            assert_eq!(pw_content, "what-are-our-options\n");
        }
        IncrementalWorld::EditedPassword { envs, .. } => {
            assert_eq!(stdout.lines().count(), 8, "Not enough commits");
            assert_eq!(
                stdout.lines().next().unwrap(),
                "Edit password for 'Manufacturers/Sokor' using libpass."
            );

            let output = Command::new("pass")
                .args(&["show", "Manufacturers/Sokor"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not read Sokor password content");
            let pw_content =
                String::from_utf8(output.stdout).expect("Cloud not read stdout as UTF-8");

            assert_eq!(pw_content, "pum-yIghoSQo'\nBetter not tell Picard about this.\nNote: Picard already knows...\n");
        }
        IncrementalWorld::RemovedPassword { envs, .. } => {
            assert_eq!(stdout.lines().count(), 8, "Not enough commits");
            assert_eq!(
                stdout.lines().next().unwrap(),
                "Remove 'Manufacturers/Sokor' from store."
            );

            let status = Command::new("pass")
                .args(&["show", "Manufacturers/Sokor"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .status()
                .expect("Could not read Sokor password content");

            assert!(!status.success(), "Password Sokor has not been removed!");
        }
        IncrementalWorld::RenamedPassword { envs, .. } => {
            assert_eq!(stdout.lines().count(), 8, "Not enough commits");
            assert_eq!(
                stdout.lines().next().unwrap(),
                "Rename 'Manufacturers/Sokor' to 'Manufacturers/None of your concern'."
            );

            let status = Command::new("pass")
                .args(&["show", "Manufacturers/None of your concern"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .status()
                .expect("Could not read Sokor password content");

            assert!(status.success(), "Sokor password has not been renamed!");
        }
        IncrementalWorld::NewPasswordAndDirectory { envs, .. } => {
            assert_eq!(stdout.lines().count(), 3, "Not enough commits");
            assert_eq!(
                stdout.lines().next().unwrap(),
                "Add password for 'Warp Nacelles/Starfleet' using libpass."
            );

            let output = Command::new("pass")
                .args(&["show", "Warp Nacelles/Starfleet"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not read Warp Nacelles/Starfleet password content");
            let pw_content =
                String::from_utf8(output.stdout).expect("Cloud not read stdout as UTF-8");

            assert_eq!(pw_content, "two-nacelles-ftw\n");
        }
        IncrementalWorld::RenamedDirectory { envs, .. } => {
            assert_eq!(stdout.lines().count(), 8, "Not enough commits");
            assert_eq!(
                stdout.lines().next().unwrap(),
                "Rename 'Entertainment/Holo Deck' to 'Entertainment/Novels'."
            );

            let status = Command::new("pass")
                .args(&["show", "Entertainment/Novels"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .status()
                .expect("Could not read Novels directory content");

            assert!(status.success(), "Novels directory has not been renamed!");
        }
        IncrementalWorld::RemovedDirectory { envs, .. } => {
            assert_eq!(stdout.lines().count(), 8, "Not enough commits");
            assert_eq!(
                stdout.lines().next().unwrap(),
                "Remove 'Entertainment' from store."
            );

            let status = Command::new("pass")
                .args(&["show", "Entertainment"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .status()
                .expect("Could not read Entertainment directory content");

            assert!(
                !status.success(),
                "Entertainment directory has not been removed!"
            );
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
    let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");
    assert_eq!(stdout, "", "Repository is not clean!");
}

#[then("pushing the commit succeeds")]
fn pushing_the_commit_succeeds(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pushed { result, envs, .. } = prev {
        result.expect("Failed to push to remote");

        let output = Command::new("pass")
            .args(&["git", "log", "origin/main..main"])
            .envs(envs.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Could not check git state");
        let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");
        let stderr = String::from_utf8(output.stderr).expect("Could not read stderr as UTF-8");
        assert_eq!(
            stderr, "",
            "Errors occurred while checking git for pushable commits!"
        );
        assert_eq!(stdout, "", "Commits are not pushed!");
    } else {
        panic!("World state is not Pushed!");
    }
}

#[then("pushing the commit fails")]
fn pushing_the_commit_fails(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pushed { result, envs, .. } = prev {
        assert!(
            result.is_err(),
            "Pushing has not failed, although it should have!"
        );

        let output = Command::new("pass")
            .args(&["git", "log", "origin/main..main"])
            .envs(envs.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Could not check git state");
        let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");
        let stderr = String::from_utf8(output.stderr).expect("Could not read stderr as UTF-8");
        assert_eq!(
            stderr, "",
            "Errors occurred while checking git for pushable commits!"
        );
        assert!(!stdout.is_empty(), "Commits are not pushed!");
    } else {
        panic!("World state is not Pushed!");
    }
}

#[then("no conflicts need to be resolved")]
fn no_conflicts_need_to_be_resolved(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pulled { mut resolving_store, envs, home, .. } = prev {
        resolving_store.with_resolver_mut(|wrapped_resolver| {
            let resolver = wrapped_resolver.take().unwrap();
            assert!(resolver.conflicted_passwords().is_empty(), "conflicted_passwords is not empty!");
            assert!(resolver.conflicted_gpg_ids().is_empty(), "conflicted_gpg_ids is not empty!");
            assert!(resolver.conflicted_plain_texts().is_empty(), "conflicted_plain_texts is not empty!");
            assert!(resolver.conflicted_binaries().is_empty(), "conflicted_binaries is not empty!");
            resolver.finish().expect("Failed to finish resolving merge conflicts");
        });
        let store = AssertUnwindSafe(resolving_store.0.into_heads().store);

        *world = IncrementalWorld::ConflictAutomaticallyResolved {
            home,
            store,
            envs,
        };
    } else {
        panic!("World state is not Pulled!");
    }
}

#[then("merge conflicts are manually resolved")]
fn merge_conflicts_are_manually_resolved(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pulled { mut resolving_store, envs, home, .. } = prev {
        resolving_store.with_resolver_mut(|wrapped_resolver| {
            let mut resolver = wrapped_resolver.take().unwrap();
            assert!(resolver.conflicted_gpg_ids().is_empty(), "conflicted_gpg_ids is not empty!");
            assert!(resolver.conflicted_plain_texts().is_empty(), "conflicted_plain_texts is not empty!");
            assert!(resolver.conflicted_binaries().is_empty(), "conflicted_binaries is not empty!");
            let mut conflicted_passwords = resolver.conflicted_passwords();
            assert_eq!(conflicted_passwords.len(), 1, "Not exactly one conflicted password!");
            for conflicted_password in conflicted_passwords.iter_mut() {
                let ancestor = conflicted_password.ancestor_password().expect("No ancestor for conflict");
                let our = conflicted_password.our_password().expect("No ours for conflict");
                let their = conflicted_password.their_password().expect("No theirs for conflict");

                let mut resolved = ancestor.clone();
                for change in ancestor.diff(&our) {
                    match change {
                        PasswordChange::Equal(_) => continue,
                        PasswordChange::Delete(_) => continue,
                        PasswordChange::Insert(lines) => {
                            for line in lines.iter() {
                                resolved.insert_line(line.my_linum, line.content);
                            }
                        },
                        PasswordChange::Replace { other_lines, .. } => {
                            for line in other_lines.iter() {
                                resolved.insert_line(line.my_linum, line.content);
                            }
                        },
                    }
                }

                for change in ancestor.diff(&their) {
                    match change {
                        PasswordChange::Equal(_) => continue,
                        PasswordChange::Delete(_) => continue,
                        PasswordChange::Insert(lines) => {
                            for line in lines.iter() {
                                resolved.insert_line(line.my_linum, line.content);
                            }
                        },
                        PasswordChange::Replace { other_lines, .. } => {
                            for line in other_lines.iter() {
                                resolved.insert_line(line.my_linum, line.content);
                            }
                        },
                    }
                }

                conflicted_password.resolve(&mut resolver, Some(resolved)).expect("Could not resolve conflict");
            }

            resolver.finish().expect("Failed to finish resolving merge conflicts");
        });
        let store = AssertUnwindSafe(resolving_store.0.into_heads().store);

        *world = IncrementalWorld::ConflictManuallyResolved {
            home,
            store,
            envs,
        };
    } else {
        panic!("World state is not Pulled!");
    }
}

#[then("binary merge conflicts are manually resolved")]
fn binary_merge_conflicts_are_manually_resolved(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pulled { mut resolving_store, envs, home, .. } = prev {
        resolving_store.with_resolver_mut(|wrapped_resolver| {
            let mut resolver = wrapped_resolver.take().unwrap();
            assert!(resolver.conflicted_gpg_ids().is_empty(), "conflicted_gpg_ids is not empty!");
            assert!(resolver.conflicted_plain_texts().is_empty(), "conflicted_plain_texts is not empty!");
            assert!(resolver.conflicted_passwords().is_empty(), "conflicted_passwords is not empty!");
            let mut conflicted_binaries = resolver.conflicted_binaries();
            assert_eq!(conflicted_binaries.len(), 1, "Not exactly one conflicted binary!");
            for conflicted_binary in conflicted_binaries.iter_mut() {
                let resolved = conflicted_binary.our_content().iter()
                    .chain(conflicted_binary.their_content().iter())
                    .chain(conflicted_binary.ancestor_content().iter())
                    .next()
                    .expect("No content available for resolution!")
                    .to_vec();

                conflicted_binary.resolve(&mut resolver, Some(&resolved)).expect("Could not resolve conflict");
            }

            resolver.finish().expect("Failed to finish resolving merge conflicts");
        });
        let store = AssertUnwindSafe(resolving_store.0.into_heads().store);

        *world = IncrementalWorld::BinaryConflictManuallyResolved {
            home,
            store,
            envs,
        };
    } else {
        panic!("World state is not Pulled!");
    }
}

#[then("plain text merge conflicts are manually resolved")]
fn plain_text_merge_conflicts_are_manually_resolved(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pulled { mut resolving_store, envs, home, .. } = prev {
        resolving_store.with_resolver_mut(|wrapped_resolver| {
            let mut resolver = wrapped_resolver.take().unwrap();
            assert!(resolver.conflicted_gpg_ids().is_empty(), "conflicted_gpg_ids is not empty!");
            assert!(resolver.conflicted_binaries().is_empty(), "conflicted_binaries is not empty!");
            assert!(resolver.conflicted_passwords().is_empty(), "conflicted_passwords is not empty!");
            let mut conflicted_plain_texts = resolver.conflicted_plain_texts();
            assert_eq!(conflicted_plain_texts.len(), 1, "Not exactly one conflicted plain text!");
            for conflicted_text in conflicted_plain_texts.iter_mut() {
                let resolved = conflicted_text.our_content().iter()
                    .chain(conflicted_text.their_content().iter())
                    .chain(conflicted_text.ancestor_content().iter())
                    .next()
                    .expect("No content available for resolution!")
                    .to_string();

                conflicted_text.resolve(&mut resolver, Some(&resolved)).expect("Could not resolve conflict");
            }

            resolver.finish().expect("Failed to finish resolving merge conflicts");
        });
        let store = AssertUnwindSafe(resolving_store.0.into_heads().store);

        *world = IncrementalWorld::TextConflictManuallyResolved {
            home,
            store,
            envs,
        };
    } else {
        panic!("World state is not Pulled!");
    }
}

#[then("gpg-id merge conflicts are manually resolved")]
fn gpg_id_merge_conflicts_are_manually_resolved(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Pulled { mut resolving_store, envs, home, .. } = prev {
        resolving_store.with_resolver_mut(|wrapped_resolver| {
            let mut resolver = wrapped_resolver.take().unwrap();
            assert!(resolver.conflicted_plain_texts().is_empty(), "conflicted_plain_texts is not empty!");
            assert!(resolver.conflicted_binaries().is_empty(), "conflicted_binaries is not empty!");
            let mut conflicted_gpg_ids = resolver.conflicted_gpg_ids();
            let mut conflicted_passwords = resolver.conflicted_passwords();
            assert_eq!(conflicted_gpg_ids.len(), 1, "Not exactly one conflicted gpg-id!");
            assert_eq!(conflicted_passwords.len(), 5, "Not exactly 5 conflicted passwords!");

            for conflicted_gpg_id in conflicted_gpg_ids.iter_mut() {
                let ours = conflicted_gpg_id.our_key_ids().expect("No key ids provided by us");
                let theirs = conflicted_gpg_id.their_key_ids().expect("No key ids provided by them");

                assert!(ours.intersection(&theirs).count() == 1, "More than one key-id difference");

                let resolved: std::collections::HashSet<_> = ours.union(&theirs).map(Clone::clone).collect();

                conflicted_gpg_id.resolve(&mut resolver, Some(&resolved))
                    .expect("Could not resolve conflict");
            }

            for conflicted_password in conflicted_passwords.iter_mut() {
                let ancestor = conflicted_password.ancestor_password().expect("No ancestor password found");
                let their = conflicted_password.their_password().expect("No their password found");
                let our = conflicted_password.our_password().expect("No our password found");

                assert!(
                    ancestor.diff(&their).iter()
                        .all(|change| matches!(change, PasswordChange::Equal(..))),
                    "Ancestor and their not equal",
                );
                assert!(
                    ancestor.diff(&our).iter()
                        .all(|change| matches!(change, PasswordChange::Equal(..))),
                    "Ancestor and our not equal",
                );
                assert!(
                    their.diff(&our).iter()
                        .all(|change| matches!(change, PasswordChange::Equal(..))),
                    "Their and our not equal",
                );

                conflicted_password.resolve(&mut resolver, Some(our))
                    .expect("Failed to resolve reencrypted password conflict");
            }

            resolver.finish().expect("Failed to finish resolving merge conflicts");
        });
        let store = AssertUnwindSafe(resolving_store.0.into_heads().store);

        *world = IncrementalWorld::GpgIdConflictManuallyResolved {
            home,
            store,
            envs,
        };
    } else {
        panic!("World state is not Pulled!");
    }
}

#[then("the remote's commits are fast-forwarded")]
fn the_remotes_commits_are_fast_forwarded(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::ConflictAutomaticallyResolved { envs, store, home } = prev {
        let output = Command::new("pass")
            .args(&["git", "log", "--pretty=format:[%an] %s", "--graph"])
            .envs(envs.clone())
            .stdout(Stdio::piped())
            .output()
            .expect("Could not check git commit");
        let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");

        assert_eq!(stdout, "* [Remote User] Add given password for Manufacturers/Sokor to store.\n\
                            * [Test User] Add given password for Entertainment/Holo Deck/Broht & Forrester to store.\n\
                            * [Test User] Add given password for Manufacturers/Sokor to store.\n\
                            * [Test User] Add given password for Manufacturers/StrutCo to store.\n\
                            * [Test User] Add given password for Phone to store.\n\
                            * [Test User] Add given password for Manufacturers/Yoyodyne to store.\n\
                            * [Test User] Configure git repository for gpg file diff.\n\
                            * [Test User] Add current contents of password store.");
        *world = IncrementalWorld::Successful { home, store, envs };
    } else {
        panic!("World state is not ConflictsAutomaticallyResolved!");
    }
}

#[then("the remote's commits are merged")]
fn the_remotes_commits_are_merged(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    match prev {
        IncrementalWorld::ConflictAutomaticallyResolved { envs, home, store } => {
            let output = Command::new("pass")
                .args(&["git", "log", "--pretty=format:[%an] %s", "--graph"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not check git commit");
            let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");

            assert_eq!(stdout, "*   [Test User] Merge origin/main into main\n\
                                |\\  \n\
                                | * [Remote User] Add given password for Manufacturers/Sokor to store.\n\
                                * | [Test User] Add password for 'Ready Room' using libpass.\n\
                                |/  \n\
                                * [Test User] Add given password for Entertainment/Holo Deck/Broht & Forrester to store.\n\
                                * [Test User] Add given password for Manufacturers/Sokor to store.\n\
                                * [Test User] Add given password for Manufacturers/StrutCo to store.\n\
                                * [Test User] Add given password for Phone to store.\n\
                                * [Test User] Add given password for Manufacturers/Yoyodyne to store.\n\
                                * [Test User] Configure git repository for gpg file diff.\n\
                                * [Test User] Add current contents of password store.");
            *world = IncrementalWorld::Successful { home, store, envs };
        },
        IncrementalWorld::ConflictManuallyResolved { envs, home, store } => {
            let output = Command::new("pass")
                .args(&["git", "log", "--pretty=format:[%an] %s", "--graph"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not check git commit");
            let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");

            assert_eq!(stdout, "*   [Test User] Merge origin/main into main\n\
                                |\\  \n\
                                | * [Remote User] Add given password for Manufacturers/Sokor to store.\n\
                                * | [Test User] Edit password for 'Manufacturers/Sokor' using libpass.\n\
                                |/  \n\
                                * [Test User] Add given password for Entertainment/Holo Deck/Broht & Forrester to store.\n\
                                * [Test User] Add given password for Manufacturers/Sokor to store.\n\
                                * [Test User] Add given password for Manufacturers/StrutCo to store.\n\
                                * [Test User] Add given password for Phone to store.\n\
                                * [Test User] Add given password for Manufacturers/Yoyodyne to store.\n\
                                * [Test User] Configure git repository for gpg file diff.\n\
                                * [Test User] Add current contents of password store.");
            *world = IncrementalWorld::Successful { home, store, envs };
        },
        IncrementalWorld::BinaryConflictManuallyResolved { envs, home, store } => {
            let output = Command::new("pass")
                .args(&["git", "log", "--pretty=format:[%an] %s", "--graph"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not check git commit");
            let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");

            assert_eq!(stdout, "*   [Test User] Merge origin/main into main\n\
                                |\\  \n\
                                | * [Remote User] Add 'Manufacturers/Sokor-Starmap' binary file to store\n\
                                * | [Test User] Add 'Manufacturers/Sokor-Starmap' to store\n\
                                |/  \n\
                                * [Test User] Add given password for Entertainment/Holo Deck/Broht & Forrester to store.\n\
                                * [Test User] Add given password for Manufacturers/Sokor to store.\n\
                                * [Test User] Add given password for Manufacturers/StrutCo to store.\n\
                                * [Test User] Add given password for Phone to store.\n\
                                * [Test User] Add given password for Manufacturers/Yoyodyne to store.\n\
                                * [Test User] Configure git repository for gpg file diff.\n\
                                * [Test User] Add current contents of password store.");
            *world = IncrementalWorld::Successful { home, store, envs };
        },
        IncrementalWorld::TextConflictManuallyResolved { envs, home, store } => {
            let output = Command::new("pass")
                .args(&["git", "log", "--pretty=format:[%an] %s", "--graph"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not check git commit");
            let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");

            assert_eq!(stdout, "*   [Test User] Merge origin/main into main\n\
                                |\\  \n\
                                | * [Remote User] Add 'Manufacturers/Sokor-Greeting' text file to store\n\
                                * | [Test User] Add 'Manufacturers/Sokor-Greeting' to store\n\
                                |/  \n\
                                * [Test User] Add given password for Entertainment/Holo Deck/Broht & Forrester to store.\n\
                                * [Test User] Add given password for Manufacturers/Sokor to store.\n\
                                * [Test User] Add given password for Manufacturers/StrutCo to store.\n\
                                * [Test User] Add given password for Phone to store.\n\
                                * [Test User] Add given password for Manufacturers/Yoyodyne to store.\n\
                                * [Test User] Configure git repository for gpg file diff.\n\
                                * [Test User] Add current contents of password store.");
            *world = IncrementalWorld::Successful { home, store, envs };
        },
        IncrementalWorld::GpgIdConflictManuallyResolved { envs, home, store } => {
            let output = Command::new("pass")
                .args(&["git", "log", "--pretty=format:[%an] %s", "--graph"])
                .envs(envs.clone())
                .stdout(Stdio::piped())
                .output()
                .expect("Could not check git commit");
            let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");

            assert_eq!(stdout, "*   [Test User] Merge origin/main into main\n\
                                |\\  \n\
                                | * [Remote User] Reencrypt password store using new GPG id test@key.email, test2@key.email.\n\
                                | * [Remote User] Set GPG id to test@key.email, test2@key.email.\n\
                                * | [Test User] Reencrypt 'Phone' as gpg-ids changed to test@key.email, test3@key.email.\n\
                                * | [Test User] Reencrypt 'Manufacturers/Yoyodyne' as gpg-ids changed to test@key.email, test3@key.email.\n\
                                * | [Test User] Reencrypt 'Manufacturers/StrutCo' as gpg-ids changed to test@key.email, test3@key.email.\n\
                                * | [Test User] Reencrypt 'Manufacturers/Sokor' as gpg-ids changed to test@key.email, test3@key.email.\n\
                                * | [Test User] Reencrypt 'Entertainment/Holo Deck/Broht & Forrester' as gpg-ids changed to test@key.email, test3@key.email.\n\
                                * | [Test User] Main GPG IDs for store set to test@key.email, test3@key.email.\n\
                                |/  \n\
                                * [Test User] Add given password for Entertainment/Holo Deck/Broht & Forrester to store.\n\
                                * [Test User] Add given password for Manufacturers/Sokor to store.\n\
                                * [Test User] Add given password for Manufacturers/StrutCo to store.\n\
                                * [Test User] Add given password for Phone to store.\n\
                                * [Test User] Add given password for Manufacturers/Yoyodyne to store.\n\
                                * [Test User] Configure git repository for gpg file diff.\n\
                                * [Test User] Add current contents of password store.");
            *world = IncrementalWorld::Successful { home, store, envs };
        },
        world => { panic!("World state is invalid: {:#?}", world); }
    }
}

#[when("a password is opened")]
fn a_password_is_opened(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { store, .. } = world {
        let entry = store
            .show("./Manufacturers/StrutCo.gpg", TraversalOrder::PreOrder)
            .unwrap()
            .next()
            .expect("Manufacturers/StrutCo password not found in password store!");

        let strutco = entry
            .password()
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

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let root = store
            .show("./", TraversalOrder::LevelOrder)
            .expect("could not get root directory of password store")
            .next()
            .expect("could not get root directory of password store")
            .directory()
            .expect("Root directory is not a directory");

        let password = root
            .password_insertion("Ready Room")
            .passphrase("what-are-our-options")
            .insert(&mut store)
            .expect("Password insertion failed");

        *world = IncrementalWorld::NewPassword {
            store,
            home,
            envs,
            password,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a password is edited")]
fn a_password_is_edited(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let password = store
            .show("Manufacturers/Sokor", TraversalOrder::LevelOrder)
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

        *world = IncrementalWorld::EditedPassword {
            store,
            home,
            envs,
            password,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a password is removed")]
fn a_password_is_removed(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let password = store
            .show("Manufacturers/Sokor", TraversalOrder::LevelOrder)
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

        *world = IncrementalWorld::RemovedPassword {
            store,
            home,
            envs,
            path,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a password is renamed")]
fn a_password_is_renamed(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let mut password = store
            .show("Manufacturers/Sokor", TraversalOrder::LevelOrder)
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

        *world = IncrementalWorld::RenamedPassword {
            store,
            home,
            envs,
            password,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("the binary file is edited")]
fn the_binary_file_is_edited(world: &mut IncrementalWorld) {
    use std::fs::File;
    use std::io::Write;

    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let content = &[0xFE, 0xED, 0xC0, 0xDE];
        let binary_file_path = store.location().join("Manufacturers/Sokor-Starmap");
        let mut binary_file = File::create(&binary_file_path).expect("Failed to create binary file");
        binary_file.write_all(content).expect("Failed to write to binary file");

        let git = store.git().expect("Store not using git");
        git.add(&[&binary_file_path]).expect("Failed to add binary file to git");
        git.commit("Add 'Manufacturers/Sokor-Starmap' to store").expect("Failed to commit binary file to git");

        *world = IncrementalWorld::Successful {
            store,
            home,
            envs,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("the text file is edited")]
fn the_text_file_is_edited(world: &mut IncrementalWorld) {
    use std::fs::File;
    use std::io::Write;

    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let content = "reboot even if system utterly broken";
        let text_file_path = store.location().join("Manufacturers/Sokor-Greeting");
        let mut text_file = File::create(&text_file_path).expect("Failed to create text file");
        text_file.write_all(content.as_bytes()).expect("Failed to write to text file");

        let git = store.git().expect("Store not using git");
        git.add(&[&text_file_path]).expect("Failed to add text file to git");
        git.commit("Add 'Manufacturers/Sokor-Greeting' to store").expect("Failed to commit text file to git");

        *world = IncrementalWorld::Successful {
            store,
            home,
            envs,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("the gpg-id of the store is edited")]
fn the_gpg_id_of_the_store_is_edited(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let root = store
            .show(".", TraversalOrder::PreOrder).expect("Root directory not found")
            .next().expect("Root directory not found")
            .directory().expect("Not a directory");

        root.make_mut(&mut store)
            .add_gpg_id(
                GpgKeyId::new("test3@key.email")
                    .expect("GPG key id test@key.email does not exist")
            ).expect("Could not add new gpg-id");

        *world = IncrementalWorld::Successful {
            store,
            home,
            envs,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a directory is created")]
fn a_directory_is_created(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { ref mut store, .. } = world {
        let root = store
            .show(".", TraversalOrder::LevelOrder)
            .expect("could not find root directory")
            .next()
            .expect("could not find root directory")
            .directory()
            .expect("Root is not a directory");

        root.directory_insertion("Warp Nacelles")
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

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let warp_nacelles = store
            .show("./Warp Nacelles", TraversalOrder::LevelOrder)
            .expect("could not get root directory of password store")
            .next()
            .expect("could not get root directory of password store")
            .directory()
            .expect("Root directory is not a directory");

        let password = warp_nacelles
            .password_insertion("Starfleet")
            .passphrase("two-nacelles-ftw")
            .insert(&mut store)
            .expect("Password insertion failed");

        *world = IncrementalWorld::NewPasswordAndDirectory {
            store,
            home,
            envs,
            password,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a directory is renamed")]
fn a_directory_is_renamed(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let mut holo_deck = store
            .show("Entertainment/Holo Deck", TraversalOrder::LevelOrder)
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

        *world = IncrementalWorld::RenamedDirectory {
            store,
            home,
            envs,
            directory,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("a directory is removed")]
fn a_directory_is_removed(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::Successful {
        mut store,
        home,
        envs,
    } = prev
    {
        let entertainment = store
            .show("Entertainment", TraversalOrder::LevelOrder)
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

        *world = IncrementalWorld::RemovedDirectory {
            store,
            home,
            envs,
            path,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("the commit is pushed to the remote")]
fn the_commit_is_pushed_to_the_remote(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    if let IncrementalWorld::NewPassword {
        mut store,
        home,
        envs,
        ..
    } = prev
    {
        let result = store
            .git()
            .expect("Store not using git")
            .push(GitRemote::UpstreamForBranch);

        *world = IncrementalWorld::Pushed {
            store,
            home,
            envs,
            result,
        };
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("changes are pulled from the remote")]
fn changes_are_pulled_from_the_remote(world: &mut IncrementalWorld) {
    let prev = std::mem::replace(world, IncrementalWorld::Initial);

    let (store, home, envs) = match prev {
        IncrementalWorld::Successful { store, home, envs, .. } => (store, home, envs),
        IncrementalWorld::NewPassword { store, home, envs, .. } => (store, home, envs),
        IncrementalWorld::EditedPassword { store, home, envs, .. } => (store, home, envs),
        _ => panic!("World state not valid: {:?}", prev),
    };

    let resolving_store = AssertUnwindSafe(ResolvingStoreBuilder {
        store: store.0,
        resolver_builder: |store: &mut Store| {
            Some(store
                .git().expect("Store not using git")
                .pull().expect("Could not pull changes from remote"))
        },
    }.build());
    *world = IncrementalWorld::Pulled {
        resolving_store,
        home,
        envs,
    };
}

#[when("the username is overridden in the git config")]
fn the_username_is_overridden_in_the_git_config(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { ref mut store, .. } = world {
        store
            .git().expect("store does not use git")
            .config().expect("could not get git config")
            .set_str("user.name", "Dynamic Test User").expect("could not set username in git");
    } else {
        panic!("World state is not Successful!");
    }
}

#[when("the email is overridden in the git config")]
fn the_email_is_overridden_in_the_git_config(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { ref mut store, .. } = world {
        store
            .git().expect("store does not use git")
            .config().expect("could not get git config")
            .set_str("user.email", "dynamic.test@key.email").expect("could not set email in git");
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the git username can be read from the repository config")]
fn the_git_username_can_be_read_from_the_repository_config(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { ref mut store, .. } = world {
        let username = store
            .git().expect("store does not use git")
            .config().expect("could not get git config")
            .get_string("user.name").expect("could not get username from git");
        assert_eq!(username, "Test User");
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the git email can be read from the repository config")]
fn the_git_email_can_be_read_from_the_repository_config(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { ref mut store, .. } = world {
        let email = store
            .git().expect("store does not use git")
            .config().expect("could not get git config")
            .get_string("user.email").expect("could not get user email from git");
        assert_eq!(email, "test@key.email");
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the git config is valid")]
fn the_git_config_is_valid(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { ref mut store, .. } = world {
        let config_valid = store
            .git().expect("store does not use git")
            .config_valid();
        assert!(
            config_valid,
            "the git config should be valid: {}, {}",
            store.git().unwrap().config().unwrap().get_entry("user.name").unwrap().value().unwrap(),
            store.git().unwrap().config().unwrap().get_entry("user.email").unwrap().value().unwrap(),
        );
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the git config is invalid")]
fn the_git_config_is_invalid(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Successful { ref mut store, .. } = world {
        let config_valid = store
            .git().expect("store does not use git")
            .config_valid();
        assert!(!config_valid, "the git config should be invalid");
    } else {
        panic!("World state is not Successful!");
    }
}

#[then("the git username for this repository is changed")]
fn the_git_username_for_this_repository_is_changed(world: &mut IncrementalWorld) {
    let envs = match world {
        IncrementalWorld::Successful { envs, .. } => envs,
        _ => panic!("World state is invalid!"),
    };

    let output = Command::new("pass")
        .args(&["git", "config", "user.name"])
        .envs(envs.clone())
        .stdout(Stdio::piped())
        .output()
        .expect("Could not get git username");
    let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");
    assert_eq!(stdout, "Dynamic Test User\n", "Username is not changed!");
}

#[then("the git email for this repository is changed")]
fn the_git_email_for_this_repository_is_changed(world: &mut IncrementalWorld) {
    let envs = match world {
        IncrementalWorld::Successful { envs, .. } => envs,
        _ => panic!("World state is invalid!"),
    };

    let output = Command::new("pass")
        .args(&["git", "config", "user.email"])
        .envs(envs.clone())
        .stdout(Stdio::piped())
        .output()
        .expect("Could not get git email");
    let stdout = String::from_utf8(output.stdout).expect("Could not read stdout as UTF-8");
    assert_eq!(stdout, "dynamic.test@key.email\n", "Email is not changed!");
}

#[then("the git status is clean")]
fn the_git_status_is_clean(world: &mut IncrementalWorld) {
    let store = match world {
        IncrementalWorld::Successful { store, .. } => store,
        _ => panic!("World state is invalid!"),
    };

    let status = store
        .git().expect("store is not using git")
        .status().expect("failed to get git status");
    assert!(status.is_clean(), "git status is not clean!");
}

#[then("the git status contains new commits")]
fn the_git_status_contains_new_commits(world: &mut IncrementalWorld) {
    let store = match world {
        IncrementalWorld::EditedPassword { store, .. } => store,
        IncrementalWorld::Successful { store, .. } => store,
        _ => panic!("World state is invalid!"),
    };

    let status = store
        .git().expect("store is not using git")
        .status().expect("failed to get git status");
    assert!(status.conflicts.is_empty(), "conflicts is not empty");
    assert!(status.workdir.is_empty(), "workdir is not empty");
    assert!(status.staging.is_empty(), "staging is not empty");
    assert_eq!(
        status.branches,
        vec![
            BranchStatus {
                branch: "main".to_owned(),
                commits_behind_remote: 0,
                commits_ahead_of_remote: 1,
            },
        ],
        "main branch not ahead of remote",
    );
}

#[then("the git status contains new commits on the remote")]
fn the_git_status_contains_new_commits_on_the_remote(world: &mut IncrementalWorld) {
    let store = match world {
        IncrementalWorld::Successful { store, .. } => store,
        _ => panic!("World state is invalid!"),
    };

    let status = store
        .git().expect("store is not using git")
        .status().expect("failed to get git status");
    assert!(status.conflicts.is_empty(), "conflicts is not empty");
    assert!(status.workdir.is_empty(), "workdir is not empty");
    assert!(status.staging.is_empty(), "staging is not empty");
    assert_eq!(
        status.branches,
        vec![
            BranchStatus {
                branch: "main".to_owned(),
                commits_behind_remote: 1,
                commits_ahead_of_remote: 0,
            },
        ],
        "main branch not ahead of remote",
    );
}

#[then("the git status contains uncommitted changes")]
fn the_git_status_contains_uncommitted_changes(world: &mut IncrementalWorld) {
    let store = match world {
        IncrementalWorld::Successful { store, .. } => store,
        _ => panic!("World state is invalid!"),
    };

    let status = store
        .git().expect("store is not using git")
        .status().expect("failed to get git status");
    assert!(status.conflicts.is_empty(), "conflicts is not empty");
    assert_eq!(status.workdir.len(), 1, "workdir has not exactly one change");
    assert!(status.staging.is_empty(), "staging is not empty");
    assert!(status.branches.is_empty(), "branches is not empty");
}

#[then("the passwords and directories are iterated in level-order form")]
fn the_passwords_and_directories_are_iterated_in_level_order_form(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);
    let entries = match prev {
        IncrementalWorld::LevelOrderTraversal { entries, .. } => entries,
        _ => panic!("World state is invalid!"),
    };

    assert_eq!(
        entries,
        vec![
            PathBuf::from(""),
            PathBuf::from("Entertainment"),
            PathBuf::from("Manufacturers"),
            PathBuf::from("Phone.gpg"),
            PathBuf::from("Entertainment/Holo Deck"),
            PathBuf::from("Manufacturers/Sokor.gpg"),
            PathBuf::from("Manufacturers/StrutCo.gpg"),
            PathBuf::from("Manufacturers/Yoyodyne.gpg"),
            PathBuf::from("Entertainment/Holo Deck/Broht & Forrester.gpg"),
        ],
    );
}

#[then("the passwords and directories are iterated in pre-order form")]
fn the_passwords_and_directories_are_iterated_in_pre_order_form(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);
    let entries = match prev {
        IncrementalWorld::PreOrderTraversal { entries, .. } => entries,
        _ => panic!("World state is invalid!"),
    };

    assert_eq!(
        entries,
        vec![
            PathBuf::from(""),
            PathBuf::from("Entertainment"),
            PathBuf::from("Entertainment/Holo Deck"),
            PathBuf::from("Entertainment/Holo Deck/Broht & Forrester.gpg"),
            PathBuf::from("Manufacturers"),
            PathBuf::from("Manufacturers/Sokor.gpg"),
            PathBuf::from("Manufacturers/StrutCo.gpg"),
            PathBuf::from("Manufacturers/Yoyodyne.gpg"),
            PathBuf::from("Phone.gpg"),
        ],
    );
}

#[then("the passwords and directories are iterated in post-order form")]
fn the_passwords_and_directories_are_iterated_in_post_order_form(world: &mut IncrementalWorld) {
    // This is needed to move out of AssertUnwindSafe
    let prev = std::mem::replace(world, IncrementalWorld::Initial);
    let entries = match prev {
        IncrementalWorld::PostOrderTraversal { entries, .. } => entries,
        _ => panic!("World state is invalid!"),
    };

    assert_eq!(
        entries,
        vec![
            PathBuf::from("Entertainment/Holo Deck/Broht & Forrester.gpg"),
            PathBuf::from("Entertainment/Holo Deck"),
            PathBuf::from("Entertainment"),
            PathBuf::from("Manufacturers/Sokor.gpg"),
            PathBuf::from("Manufacturers/StrutCo.gpg"),
            PathBuf::from("Manufacturers/Yoyodyne.gpg"),
            PathBuf::from("Manufacturers"),
            PathBuf::from("Phone.gpg"),
            PathBuf::from(""),
        ],
    );
}
