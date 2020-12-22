use std::collections::HashMap;
use std::process::{Stdio, Command};
use std::io::Write;

use cucumber_rust::given;

use crate::world::IncrementalWorld;

#[given("no password store exists")]
fn no_password_store_exists(world: &mut IncrementalWorld) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("no-password-store").unwrap();
    } else {
        panic!("World state is not Initial!");
    }
}

#[given(regex = "a password store exists(.*)")]
fn a_password_store_exists(world: &mut IncrementalWorld, location: String) {
    if let IncrementalWorld::Initial = world {
        *world = IncrementalWorld::clean_env("password-store").unwrap();
        if let IncrementalWorld::Prepared { envs, key_id, home, .. } = world {
            match location.as_str() {
                "" => {},
                " at a manually provided location" => {
                    let path = home.path().join("custom-password-store");
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
