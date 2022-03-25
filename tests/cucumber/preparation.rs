use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::fs::File;

use cucumber::given;

use crate::world::IncrementalWorld;

#[given("no password store exists")]
fn no_password_store_exists(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("no-password-store").unwrap();
    }

    if let IncrementalWorld::Prepared { .. } = world {
        // nop
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given(regex = "a password store exists(.*)")]
fn a_password_store_exists(world: &mut IncrementalWorld, location: String) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("password-store").unwrap();
    }

    if let IncrementalWorld::Prepared {
        envs, key_id, home, ..
    } = world
    {
        match location.as_str() {
            "" => {}
            " at a manually provided location" => {
                let path = home.path().join("custom-password-store");
                envs.insert(
                    "PASSWORD_STORE_DIR".to_owned(),
                    format!("{}", path.display()),
                );
            }
            _ => {
                panic!("Invalid location '{}' for password insertion!", location,);
            }
        };

        let status = Command::new("pass")
            .args(&["init", key_id.as_str()])
            .envs(envs)
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to initialize pass repository!");

        println!("Password store initialized for {}", key_id);
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the password store uses git")]
fn the_password_store_uses_git(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, .. } = world {
        let status = Command::new("git")
            .args(&["config", "--global", "user.name", "Test User"])
            .envs(envs.clone())
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set username in git config");

        let status = Command::new("git")
            .args(&["config", "--global", "user.email", "test@key.email"])
            .envs(envs.clone())
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set email in git config");

        let status = Command::new("git")
            .args(&["config", "--global", "init.defaultBranch", "main"])
            .envs(envs.clone())
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set default branch name in git config");

        let status = Command::new("pass")
            .args(&["git", "init"])
            .envs(envs.clone())
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(
            status.success(),
            "Failed to initialize git in pass repository!"
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

fn insert_password(envs: &HashMap<String, String>, name: &str, content: &str) {
    let mut child = Command::new("pass")
        .args(&["insert", "--multiline", name])
        .env_clear()
        .envs(envs)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(content.as_bytes()).unwrap();
    let status = child.wait().unwrap();
    assert!(
        status.success(),
        "Failed to insert password into pass repository!"
    );
}

#[given(regex = "passwords are stored in the password store")]
fn passwords_are_stored_in_the_password_store(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, home, .. } = world {
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
        insert_password(envs, "Manufacturers/StrutCo", "i*aint*got*no*tape");
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

        let store_path = envs
            .get("PASSWORD_STORE_DIR")
            .map(|s| s.to_owned())
            .unwrap_or(format!("{}", home.path().join(".password-store").display()));
        Command::new("tree")
            .args(&["--noreport", &store_path])
            .status()
            .unwrap();
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the repository has a remote")]
fn the_repository_has_a_remote(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, home, .. } = world {
        let password_store_dir = if let Ok(path) = std::env::var("PASSWORD_STORE_DIR") {
            path.into()
        } else if let Some(path) = envs.get("PASSWORD_STORE_DIR") {
            path.into()
        } else {
            home.path().join(".password-store")
        };
        let password_store_remote = home.path().join("password-store-remote");

        let status = Command::new("git")
            .arg("clone")
            .arg("--bare")
            .arg(&password_store_dir)
            .arg(&password_store_remote)
            .envs(envs.clone())
            .status()
            .expect("Failed to prepare fake remote");
        assert!(status.success(), "Failed to prepare fake remote");

        let status = Command::new("git")
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg(password_store_remote.display().to_string())
            .envs(envs.clone())
            .current_dir(&password_store_dir)
            .status()
            .expect("failed to set origin in password store");
        assert!(status.success(), "Failed to add remote to repository");

        let status = Command::new("git")
            .arg("fetch")
            .envs(envs.clone())
            .current_dir(&password_store_dir)
            .status()
            .expect("failed to git fetch from remote");
        assert!(
            status.success(),
            "Failed to fetch from the repository's remote"
        );

        let status = Command::new("git")
            .arg("branch")
            .arg("--set-upstream-to=origin/main")
            .envs(envs.clone())
            .current_dir(&password_store_dir)
            .status()
            .expect("failed to set tracking branch");
        assert!(status.success(), "Failed to set tracking branch");
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the repository's remote contains new commits")]
fn the_repositorys_remote_contains_new_commits(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, home, .. } = world {
        let password_store_dir = if let Ok(path) = std::env::var("PASSWORD_STORE_DIR") {
            path.into()
        } else if let Some(path) = envs.get("PASSWORD_STORE_DIR") {
            path.into()
        } else {
            home.path().join(".password-store")
        };
        let password_store_remote = home.path().join("password-store-remote");

        let password_store_remote_temp_checkout = tempfile::Builder::new()
            .prefix("libpass-remote-temp-checkout_")
            .tempdir()
            .expect("Failed to create temporary checkout directory");

        let status = Command::new("git")
            .arg("clone")
            .arg(&password_store_remote)
            .arg(
                &password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .envs(envs.clone())
            .status()
            .expect("Failed to prepare temporary checkout");
        assert!(status.success(), "Failed to prepare temporary checkout");

        let status = Command::new("pass")
            .args(&["git", "config", "user.name", "Remote User"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set username in git config");

        let status = Command::new("pass")
            .args(&["git", "config", "user.email", "remote@key.email"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set email in git config");

        let content = "pum-yIghoSQo'\nBetter not tell Picard about this.\nPicard here: Let's talk about this later...\n";
        let mut child = Command::new("pass")
            .args(&["insert", "--force", "--multiline", "Manufacturers/Sokor"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(content.as_bytes()).unwrap();
        let output = child.wait_with_output().unwrap();
        let stderr = String::from_utf8(output.stderr).expect("Could not read stderr as UTF-8");
        assert_eq!(stderr, "");
        assert!(
            output.status.success(),
            "Failed to insert password into pass repository!"
        );

        let output = Command::new("pass")
            .arg("git")
            .arg("push")
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .output()
            .expect("Failed to push changes to remote!");
        assert!(output.status.success(), "Failed to push changes to remote!");

        let status = Command::new("git")
            .arg("fetch")
            .envs(envs.clone())
            .current_dir(&password_store_dir)
            .status()
            .expect("failed to git fetch from remote");
        assert!(
            status.success(),
            "Failed to fetch from the repository's remote"
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the repository's remote contains new commits of a binary file")]
fn the_repositorys_remote_contains_new_commits_of_a_binary_file(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, home, .. } = world {
        let password_store_dir = if let Ok(path) = std::env::var("PASSWORD_STORE_DIR") {
            path.into()
        } else if let Some(path) = envs.get("PASSWORD_STORE_DIR") {
            path.into()
        } else {
            home.path().join(".password-store")
        };
        let password_store_remote = home.path().join("password-store-remote");

        let password_store_remote_temp_checkout = tempfile::Builder::new()
            .prefix("libpass-remote-temp-checkout_")
            .tempdir()
            .expect("Failed to create temporary checkout directory");

        let status = Command::new("git")
            .arg("clone")
            .arg(&password_store_remote)
            .arg(
                &password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .envs(envs.clone())
            .status()
            .expect("Failed to prepare temporary checkout");
        assert!(status.success(), "Failed to prepare temporary checkout");

        let status = Command::new("pass")
            .args(&["git", "config", "user.name", "Remote User"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set username in git config");

        let status = Command::new("pass")
            .args(&["git", "config", "user.email", "remote@key.email"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set email in git config");

        let content = &[0xDE, 0xAD, 0xBE, 0xEF];
        let binary_file_path = password_store_remote_temp_checkout.path()
            .join("Manufacturers/Sokor-Starmap");
        let mut binary_file = File::create(&binary_file_path).expect("Failed to create binary file");
        binary_file.write_all(content).expect("Failed to write to binary file");

        let status = Command::new("git")
            .arg("add")
            .arg(binary_file_path)
            .envs(envs.clone())
            .current_dir(&password_store_remote_temp_checkout)
            .status()
            .expect("Failed to add binary file to git");
        assert!(
            status.success(),
            "Failed to add binary file to git"
        );

        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Add 'Manufacturers/Sokor-Starmap' binary file to store")
            .envs(envs.clone())
            .current_dir(&password_store_remote_temp_checkout)
            .status()
            .expect("Failed to commit binary file to git");
        assert!(
            status.success(),
            "Failed to commit binary file to git"
        );

        let output = Command::new("pass")
            .arg("git")
            .arg("push")
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .output()
            .expect("Failed to push changes to remote!");
        assert!(output.status.success(), "Failed to push changes to remote!");

        let status = Command::new("git")
            .arg("fetch")
            .envs(envs.clone())
            .current_dir(&password_store_dir)
            .status()
            .expect("failed to git fetch from remote");
        assert!(
            status.success(),
            "Failed to fetch from the repository's remote"
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the repository's remote contains new commits of a text file")]
fn the_repositorys_remote_contains_new_commits_of_a_text_file(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, home, .. } = world {
        let password_store_dir = if let Ok(path) = std::env::var("PASSWORD_STORE_DIR") {
            path.into()
        } else if let Some(path) = envs.get("PASSWORD_STORE_DIR") {
            path.into()
        } else {
            home.path().join(".password-store")
        };
        let password_store_remote = home.path().join("password-store-remote");

        let password_store_remote_temp_checkout = tempfile::Builder::new()
            .prefix("libpass-remote-temp-checkout_")
            .tempdir()
            .expect("Failed to create temporary checkout directory");

        let status = Command::new("git")
            .arg("clone")
            .arg(&password_store_remote)
            .arg(
                &password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .envs(envs.clone())
            .status()
            .expect("Failed to prepare temporary checkout");
        assert!(status.success(), "Failed to prepare temporary checkout");

        let status = Command::new("pass")
            .args(&["git", "config", "user.name", "Remote User"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set username in git config");

        let status = Command::new("pass")
            .args(&["git", "config", "user.email", "remote@key.email"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set email in git config");

        let content = "raising elephants is so utterly boring";
        let text_file_path = password_store_remote_temp_checkout.path()
            .join("Manufacturers/Sokor-Greeting");
        let mut text_file = File::create(&text_file_path).expect("Failed to create text file");
        text_file.write_all(content.as_bytes()).expect("Failed to write to text file");

        let status = Command::new("git")
            .arg("add")
            .arg(text_file_path)
            .envs(envs.clone())
            .current_dir(&password_store_remote_temp_checkout)
            .status()
            .expect("Failed to add text file to git");
        assert!(
            status.success(),
            "Failed to add text file to git"
        );

        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Add 'Manufacturers/Sokor-Greeting' text file to store")
            .envs(envs.clone())
            .current_dir(&password_store_remote_temp_checkout)
            .status()
            .expect("Failed to commit text file to git");
        assert!(
            status.success(),
            "Failed to commit text file to git"
        );

        let output = Command::new("pass")
            .arg("git")
            .arg("push")
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .output()
            .expect("Failed to push changes to remote!");
        assert!(output.status.success(), "Failed to push changes to remote!");

        let status = Command::new("git")
            .arg("fetch")
            .envs(envs.clone())
            .current_dir(&password_store_dir)
            .status()
            .expect("failed to git fetch from remote");
        assert!(
            status.success(),
            "Failed to fetch from the repository's remote"
        );
    } else {
        panic!("World state is not Prepared!");
    }
}

#[given("the repository's remote contains new commits changing the gpg-id")]
fn the_repositorys_remote_contains_new_commits_changing_the_gpg_id(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Prepared { envs, home, .. } = world {
        let password_store_dir = if let Ok(path) = std::env::var("PASSWORD_STORE_DIR") {
            path.into()
        } else if let Some(path) = envs.get("PASSWORD_STORE_DIR") {
            path.into()
        } else {
            home.path().join(".password-store")
        };
        let password_store_remote = home.path().join("password-store-remote");

        let password_store_remote_temp_checkout = tempfile::Builder::new()
            .prefix("libpass-remote-temp-checkout_")
            .tempdir()
            .expect("Failed to create temporary checkout directory");

        let status = Command::new("git")
            .arg("clone")
            .arg(&password_store_remote)
            .arg(
                &password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .envs(envs.clone())
            .status()
            .expect("Failed to prepare temporary checkout");
        assert!(status.success(), "Failed to prepare temporary checkout");

        let status = Command::new("pass")
            .args(&["git", "config", "user.name", "Remote User"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set username in git config");

        let status = Command::new("pass")
            .args(&["git", "config", "user.email", "remote@key.email"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "Failed to set email in git config");

        let output = Command::new("pass")
            .args(&["init", "test@key.email", "test2@key.email"])
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .output()
            .expect("Failed to add new key to password store");
        assert!(output.status.success(), "Failed to add new key to password store!");

        let output = Command::new("pass")
            .arg("git")
            .arg("push")
            .envs(envs.clone())
            .env(
                "PASSWORD_STORE_DIR",
                password_store_remote_temp_checkout
                    .path()
                    .display()
                    .to_string(),
            )
            .output()
            .expect("Failed to push changes to remote!");
        assert!(output.status.success(), "Failed to push changes to remote!");

        let status = Command::new("git")
            .arg("fetch")
            .envs(envs.clone())
            .current_dir(&password_store_dir)
            .status()
            .expect("failed to git fetch from remote");
        assert!(
            status.success(),
            "Failed to fetch from the repository's remote"
        );
    } else {
        panic!("World state is not Prepared!");
    }
}
