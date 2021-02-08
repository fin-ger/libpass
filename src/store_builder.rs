use crate::{Store, StoreError, Location, PassphraseProvider, Umask, SigningKey};

#[derive(Debug, Clone)]
pub struct StoreBuilder {
    location: Location,
    passphrase_provider: PassphraseProvider,
    umask: Umask,
    signing_key: SigningKey,
}

impl Default for StoreBuilder {
    fn default() -> Self {
        Self {
            location: Location::Automatic,
            passphrase_provider: PassphraseProvider::SystemAgent,
            umask: Umask::Automatic,
            signing_key: SigningKey::Automatic,

        }
    }
}

impl StoreBuilder {
    pub fn location<L>(self, location: L) -> Self
    where
        L: Into<Location>
    {
        Self {
            location: location.into(),
            ..self
        }
    }

    pub fn passphrase_provider<P>(self, passphrase_provider: P) -> Self
    where
        P: Into<PassphraseProvider>
    {
        Self {
            passphrase_provider: passphrase_provider.into(),
            ..self
        }
    }

    pub fn umask<U>(self, umask: U) -> Self
    where
        U: Into<Umask>
    {
        Self {
            umask: umask.into(),
            ..self
        }
    }

    pub fn signing_key<K>(self, signing_key: K) -> Self
    where
        K: Into<SigningKey>
    {
        Self {
            signing_key: signing_key.into(),
            ..self
        }
    }

    pub fn init(self, gpg_id: &str) -> Result<Store, StoreError> {
        Store::init(
            self.location,
            self.passphrase_provider,
            self.umask,
            self.signing_key,
            gpg_id,
        )
    }

    pub fn open(self) -> Result<Store, StoreError> {
        Store::open(
            self.location,
            self.passphrase_provider,
            self.umask,
            self.signing_key,
        )
    }
}
