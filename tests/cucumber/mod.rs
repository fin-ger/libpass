mod content;
mod creation;
mod preparation;
mod world;

use anyhow::Context as AnyhowContext;
use cucumber::{World, StatsWriter};
use gpgme::{Context, CreateKeyFlags, PassphraseRequest, PinentryMode, Protocol};
use std::{env, fs, io::Write, path::Path, process::Command, time::Duration};
use world::IncrementalWorld;

const DIR: bool = true;
const PW: bool = false;

#[tokio::main]
async fn main() {
    let pgp_home = env::temp_dir().join("libpass-pgp-home");
    fs::remove_dir_all(&pgp_home).ok();
    fs::create_dir_all(&pgp_home).expect("Could not create temporary home folder for PGP home");
    initialize_pgp_home(&pgp_home).expect("Failed to initialize PGP home");

    let summary = IncrementalWorld::cucumber()
        .max_concurrent_scenarios(Some(num_cpus::get()))
        .run("./features")
        .await;

    fs::remove_dir_all(&pgp_home).expect("Could not cleanup PGP home");

    if summary.execution_has_failed() {
        let failed_steps = summary.failed_steps();
        let parsing_errors = summary.parsing_errors();
        panic!(
            "{} step{} failed, {} parsing error{}",
            failed_steps,
            (failed_steps != 1).then(|| "s").unwrap_or_default(),
            parsing_errors,
            (parsing_errors != 1).then(|| "s").unwrap_or_default(),
        );
    }
}

fn create_dsa_elgamal_keypair(ctx: &mut Context, name: &str) -> anyhow::Result<()> {
    let result = ctx.create_key_with_flags(
        name,
        "DSA",
        Duration::from_secs(0),
        CreateKeyFlags::SIGN | CreateKeyFlags::NOEXPIRE,
    ).context("Failed to create signing GPG key")?;
    let key = ctx.get_key(
        result.fingerprint().ok()
            .context("fingerprint not utf-8")?
    ).context("Could not find key")?;
    ctx.create_subkey_with_flags(
        &key,
        "ELG",
        Duration::from_secs(0),
        CreateKeyFlags::ENCR | CreateKeyFlags::NOEXPIRE,
    ).context("Failed to create encryption GPG key")?;

    Ok(())
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
        .args(&[
            &format!("{}/.gnupg", home.display()),
            "-type",
            "f",
            "-exec",
            "chmod",
            "600",
            "{}",
            ";",
        ])
        .status()
        .unwrap();
    Command::new("find")
        .args(&[
            &format!("{}/.gnupg", home.display()),
            "-type",
            "d",
            "-exec",
            "chmod",
            "700",
            "{}",
            ";",
        ])
        .status()
        .unwrap();

    let mut ctx = Context::from_protocol(Protocol::OpenPgp)
        .context("Could not create GPG context")?
        .set_passphrase_provider(|_req: PassphraseRequest, out: &mut dyn Write| {
            out.write_all(b"test1234\n").unwrap();
            Ok(())
        });
    ctx.set_pinentry_mode(PinentryMode::Loopback)
        .context("Could not set pinentry mode in GPG")?;

    create_dsa_elgamal_keypair(&mut ctx, "Test Key <test@key.email>")
        .context("Failed to create test@key.email keypairs")?;
    create_dsa_elgamal_keypair(&mut ctx, "Test 2 Key <test2@key.email>")
        .context("Failed to create test2@key.email keypairs")?;
    create_dsa_elgamal_keypair(&mut ctx, "Test 3 Key <test3@key.email>")
        .context("Failed to create test3@key.email keypairs")?;
    let key = ctx.get_secret_key("test@key.email")?;

    for subkey in key.subkeys() {
        Command::new("/usr/libexec/gpg-preset-passphrase")
            .args(&["--preset", "--passphrase", "test1234", subkey.keygrip().unwrap()])
            .env_clear()
            .env("GNUPGHOME", home.join(".gnupg"))
            .output()
            .expect("Could not set passphrase in gpg-agent");
    }

    println!("GPG environment initialized for all tests");

    Ok(())
}
