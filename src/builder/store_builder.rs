use crate::{Location, PassphraseProvider, SigningKey, Sorting, Store, StoreError, Umask};

#[derive(Debug, Clone)]
pub struct StoreBuilder {
    location: Location,
    passphrase_provider: PassphraseProvider,
    umask: Umask,
    signing_key: SigningKey,
    sorting: Sorting,
}

impl Default for StoreBuilder {
    fn default() -> Self {
        Self {
            location: Location::Automatic,
            passphrase_provider: PassphraseProvider::SystemAgent,
            umask: Umask::Automatic,
            signing_key: SigningKey::Automatic,
            sorting: Sorting::NONE,
        }
    }
}

impl StoreBuilder {
    pub fn location<L>(&mut self, location: L) -> &mut Self
    where
        L: Into<Location>,
    {
        self.location = location.into();
        self
    }

    pub fn passphrase_provider<P>(&mut self, passphrase_provider: P) -> &mut Self
    where
        P: Into<PassphraseProvider>,
    {
        self.passphrase_provider = passphrase_provider.into();
        self
    }

    pub fn umask<U>(&mut self, umask: U) -> &mut Self
    where
        U: Into<Umask>,
    {
        self.umask = umask.into();
        self
    }

    pub fn signing_key<K>(&mut self, signing_key: K) -> &mut Self
    where
        K: Into<SigningKey>,
    {
        self.signing_key = signing_key.into();
        self
    }

    pub fn sorting<S>(&mut self, sorting: S) -> &mut Self
    where
        S: Into<Sorting>,
    {
        self.sorting = sorting.into();
        self
    }

    pub fn init(&self, gpg_id: &str) -> Result<Store, StoreError> {
        Store::init(
            self.location.clone(),
            self.passphrase_provider.clone(),
            self.umask.clone(),
            self.signing_key.clone(),
            self.sorting,
            gpg_id,
        )
    }

    pub fn open(&self) -> Result<Store, StoreError> {
        Store::open(
            self.location.clone(),
            self.passphrase_provider.clone(),
            self.umask.clone(),
            self.signing_key.clone(),
            self.sorting,
        )
    }
}
