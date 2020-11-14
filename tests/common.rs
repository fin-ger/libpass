use std::process::Command;
use std::path::Path;
use std::env;
use std::io::Write;

use tempdir::TempDir;
use gpgme::{Context, Protocol, PassphraseRequest, PinentryMode};

pub struct Setup {
    home: TempDir,
}

pub fn setup(name: &str, scripts: &[&str]) -> Setup {
    let home = TempDir::new(&format!("libpass-{}", name)).expect("Could not create temporary home folder");
    env::set_var("HOME", home.path());

    let mut store_dir = home.path().to_owned();
    store_dir.push("password-store");
    env::set_var("PASSWORD_STORE_DIR", &store_dir);

    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap()
        .set_passphrase_provider(|_req: PassphraseRequest, out: &mut dyn Write| {
            out.write_all(b"test1234").unwrap();
            Ok(())
        });
    ctx.set_pinentry_mode(PinentryMode::Loopback).unwrap();
    let result = ctx.create_key("Test Key <test@key.email>", "default", None).unwrap();
    let key_id = result.fingerprint().unwrap();
    env::set_var("PASSWORD_STORE_KEY", key_id);

    for script in scripts {
        let output = Command::new(Path::new(&format!("./tests/setups/{}.sh", script)))
            .env("HOME", home.path())
            .env("PASSWORD_STORE_DIR", &store_dir)
            .env("PASSWORD_STORE_KEY", key_id)
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    Setup {
        home,
    }
}

pub fn teardown(setup: Setup) {
    setup.home.close().unwrap();
}
