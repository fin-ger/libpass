mod content;
mod creation;
mod preparation;
mod world;

use anyhow::Context as AnyhowContext;
use cucumber::{FailureWriter, WorldInit};
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

    ctx.create_key_with_flags(
        "Test Key <test@key.email>",
        "default",
        Duration::from_secs(0),
        CreateKeyFlags::SIGN | CreateKeyFlags::ENCR | CreateKeyFlags::NOEXPIRE,
    )
    .context("Failed to create GPG key")?;
    ctx.create_key_with_flags(
        "Test Key 2 <test2@key.email>",
        "default",
        Duration::from_secs(0),
        CreateKeyFlags::SIGN | CreateKeyFlags::ENCR | CreateKeyFlags::NOEXPIRE,
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
