use crate::{Store, StoreError, Location, PassphraseProvider};

pub struct StoreBuilder {
    location: Location,
    passphrase_provider: PassphraseProvider,
    umask: u32,
}

impl Default for StoreBuilder {
    fn default() -> Self {
        Self {
            location: Location::Automatic,
            passphrase_provider: PassphraseProvider::SystemAgent,
            umask: 0o077,
        }
    }
}

impl StoreBuilder {
    pub fn location(self, location: Location) -> Self {
        Self {
            location,
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

    pub fn umask(self, umask: u32) -> Self {
        Self {
            umask,
            ..self
        }
    }

    pub fn init(self, gpg_id: &str) -> Result<Store, StoreError> {
        Store::init(
            self.location,
            self.passphrase_provider,
            self.umask,
            gpg_id,
        )
    }

    pub fn open(self) -> Result<Store, StoreError> {
        Store::open(
            self.location,
            self.passphrase_provider,
            self.umask,
        )
    }
}
