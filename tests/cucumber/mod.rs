mod content;
mod creation;
mod preparation;
mod world;

use anyhow::Context as AnyhowContext;
use cucumber::WorldInit;
use gpgme::{Context, CreateKeyFlags, PassphraseRequest, PinentryMode, Protocol};
use std::{env, fs, io::Write, path::Path, process::Command};
use world::IncrementalWorld;

const DIR: bool = true;
const PW: bool = false;

fn main() {
    let pgp_home = env::temp_dir().join("libpass-pgp-home");
    fs::remove_dir_all(&pgp_home).ok();
    fs::create_dir_all(&pgp_home).expect("Could not create temporary home folder for PGP home");
    initialize_pgp_home(&pgp_home).expect("Failed to initialize PGP home");

    // You may choose any executor you like (Tokio, async-std, etc)
    // You may even have an async main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    futures::executor::block_on(IncrementalWorld::run("./features"));

    fs::remove_dir_all(&pgp_home).expect("Could not cleanup PGP home");
}

fn initialize_pgp_home(home: &Path) -> anyhow::Result<()> {
    env::remove_var("HOME");
    env::set_var("GNUPGHOME", home.join(".gnupg"));

    fs::create_dir_all(home.join(".gnupg"))?;
    let mut gpg_agent_conf = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(home.join(".gnupg").join("gpg-agent.conf"))?;
    gpg_agent_conf.write_all("allow-preset-passphrase\n".as_bytes())?;
    gpg_agent_conf.flush()?;
    Command::new("find")
        .args(&[&format!("{}/.gnupg", home.display()), "-type", "f", "-exec", "chmod", "600", "{}", ";"])
        .status().unwrap();
    Command::new("find")
        .args(&[&format!("{}/.gnupg", home.display()), "-type", "d", "-exec", "chmod", "700", "{}", ";"])
        .status().unwrap();

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)
        .context("Could not create GPG context")?
        .set_passphrase_provider(|_req: PassphraseRequest, out: &mut dyn Write| {
            out.write_all(b"test1234\n").unwrap();
            Ok(())
        });
    ctx.set_pinentry_mode(PinentryMode::Loopback)
        .context("Could not set pinentry mode in GPG")?;

    ctx.create_key_with_flags(
        "Test Key <test@key.email>",
        "default",
        None,
        CreateKeyFlags::SIGN | CreateKeyFlags::ENCR,
    )
    .context("Failed to create GPG key")?;
    ctx.create_key_with_flags(
        "Test Key 2 <test2@key.email>",
        "default",
        None,
        CreateKeyFlags::SIGN | CreateKeyFlags::ENCR,
    )
    .context("Failed to create GPG key")?;
    let key = ctx.get_secret_key("test@key.email")?;
    let keygrip = key.subkeys().next().unwrap().keygrip().unwrap();

    Command::new("/usr/libexec/gpg-preset-passphrase")
        .args(&["--preset", "--passphrase", "test1234", keygrip])
        .env_clear()
        .env("GNUPGHOME", home.join(".gnupg"))
        .output()
        .expect("Could not set passphrase in gpg-agent");

    println!("GPG environment initialized for all tests");

    Ok(())
}
