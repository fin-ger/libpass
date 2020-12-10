use std::env;
use std::io::Write;
use anyhow::{bail, Context as AnyhowContext};
use tempdir::TempDir;
use gpgme::{Context, Protocol, PassphraseRequest, PinentryMode};
use tar::Builder;
use base64::write::EncoderStringWriter;
use zstd::stream::write::Encoder;

fn create_pgp_id() -> anyhow::Result<String> {
    let mut ctx = Context::from_protocol(Protocol::OpenPgp)
        .context("Could not create GPG context")?
        .set_passphrase_provider(|_req: PassphraseRequest, out: &mut dyn Write| {
            out.write_all(b"test1234").unwrap();
            Ok(())
        });
    ctx.set_pinentry_mode(PinentryMode::Loopback)
        .context("Could not set pinentry mode in GPG")?;

    let result = ctx.create_key("Test Key <test@key.email>", "default", None)
        .context("Failed to create GPG key")?;
    let key_id = match result.fingerprint() {
        Ok(key_id) => key_id,
        Err(_err) => bail!("Could not get fingerprint of new GPG key"),
    };

    Ok(key_id.to_string())
}

#[test]
#[ignore]
fn generate_gnupg_home() -> anyhow::Result<()> {
    let home = TempDir::new("libpass-tests")
        .context(format!("Could not create temporary home folder"))?;
    env::set_var("HOME", home.path());
    let key_id = create_pgp_id()?;
    let mut b64_tar = String::new();
    let encoder = EncoderStringWriter::from(&mut b64_tar, base64::STANDARD);
    let compresser = Encoder::new(encoder, 21)?;
    let mut archive = Builder::new(compresser.auto_finish());
    archive.append_dir_all("", home.path())?;
    archive.finish()?;
    drop(archive);

    println!("let key_id = \"{}\";", key_id);
    println!("let b64_tar = \"{}\";", b64_tar);

    Ok(())
}

