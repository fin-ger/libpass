mod store;
mod directory;
mod password;
mod entries;
mod iterators;

pub use store::*;
pub use directory::*;
pub use password::*;
pub use entries::*;
pub use iterators::*;

#[cfg(test)]
mod tests {
    use gpgme::{Context, KeyListMode, Protocol};
    use anyhow::Result;
    use crate::{Store, Location, Sorting, Directory};

    fn print_dir(dir: &Directory<'_>) {
        println!("Passwords:");
        for password in dir.passwords() {
            println!("  {}: {}", password.path().display(), password.name());
        }

        println!("Directories:");
        for dir in dir.directories() {
            println!("  {}: {}", dir.path().display(), dir.name());
        }

        println!("Entries:");
        for entry in dir.entries() {
            let kind = if entry.is_password() { "PW" } else { "DIR" };
            println!("  {} {}: {}", kind, entry.path().display(), entry.name());
        }

        println!();

        for dir in dir.directories() {
            print_dir(&dir);
        }
    }

    #[test]
    fn smoke() -> Result<()> {
        let store = Store::open(Location::Automatic)?;
        let content = store.content(Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST);
        print_dir(&content);
        Ok(())
    }

    #[test]
    fn no_sorting() -> Result<()> {
        let store = Store::open(Location::Automatic)?;
        let content = store.content(Sorting::NONE);
        print_dir(&content);
        Ok(())
    }

    #[test]
    fn run() -> Result<()> {
        let mode = KeyListMode::empty();
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
        ctx.set_key_list_mode(mode)?;
        let mut keys = ctx.secret_keys()?;
        for key in keys.by_ref().filter_map(|x| x.ok()) {
            println!("keyid   : {}", key.id().unwrap_or("?"));
            println!("fpr     : {}", key.fingerprint().unwrap_or("?"));
            println!(
                "caps    : {}{}{}{}",
                if key.can_encrypt() { "e" } else { "" },
                if key.can_sign() { "s" } else { "" },
                if key.can_certify() { "c" } else { "" },
                if key.can_authenticate() { "a" } else { "" }
            );
            println!(
                "flags   :{}{}{}{}{}{}",
                if key.has_secret() { " secret" } else { "" },
                if key.is_revoked() { " revoked" } else { "" },
                if key.is_expired() { " expired" } else { "" },
                if key.is_disabled() { " disabled" } else { "" },
                if key.is_invalid() { " invalid" } else { "" },
                if key.is_qualified() { " qualified" } else { "" }
            );
            for (i, user) in key.user_ids().enumerate() {
                println!("userid {}: {}", i, user.id().unwrap_or("[none]"));
                println!("valid  {}: {:?}", i, user.validity())
            }
            println!("");
        }

        Ok(())
    }
}
