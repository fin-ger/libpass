use std::{env, io, path::PathBuf};

use tempfile::PersistError;
use thiserror::Error;

pub struct StoreErrors<'a> {
    iter: std::slice::Iter<'a, StoreError>,
}

impl<'a> StoreErrors<'a> {
    pub(crate) fn new(errors: &'a Vec<StoreError>) -> Self {
        Self {
            iter: errors.into_iter(),
        }
    }
}

impl<'a> Iterator for StoreErrors<'a> {
    type Item = &'a StoreError;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Could not open or modify entry {0} in password store")]
    Io(String, #[source] io::Error),
    #[error("Password store path is not a directory: {0}")]
    NoDirectory(PathBuf),
    #[error("environment variable {0} is not set")]
    EnvVar(String, #[source] env::VarError),
    #[error("Cannot find home directory for current user: {0}")]
    NoHome(String, #[source] Box<StoreError>),
    #[error("Given path is not contained in the password store: {0}")]
    NotInStore(PathBuf),
    #[error("GPG operation failed: {0}")]
    Gpg(String, #[source] gpgme::Error),
    #[error("No GPG IDs found for '{0}' and all its parent directories")]
    NoGpgId(String),
    #[error("Invalid passphrase index {0}")]
    PassphraseIndex(usize),
    #[error("Generating passphrase failed: {0}")]
    PassphraseGeneration(&'static str),
    #[error("Failed to persist temporary passphrase for {0}")]
    PassphrasePersist(String, #[source] PersistError),
    #[error("The git operation {0} failed")]
    GitError(String, #[source] git2::Error),

    #[cfg(feature = "parsed-passwords")]
    #[error("Failed to parse password content for {0}")]
    Parse(String, #[source] Box<dyn std::error::Error + Send + Sync>),
    #[cfg(feature = "parsed-passwords")]
    #[error("Line at position {0} is not a comment")]
    PasswordLineNotAComment(Position),
    #[cfg(feature = "parsed-passwords")]
    #[error("Line at position {0} is not an entry")]
    PasswordLineNotAnEntry(Position),
}

pub(crate) trait IntoStoreError<T> {
    fn with_store_error<S: Into<String>>(self: Self, details: S) -> Result<T, StoreError>;
}

impl<T> IntoStoreError<T> for Result<T, io::Error> {
    fn with_store_error<S: Into<String>>(self: Self, path: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::Io(path.into(), err))
    }
}

impl<T> IntoStoreError<T> for Result<T, env::VarError> {
    fn with_store_error<S: Into<String>>(self: Self, env_var_name: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::EnvVar(env_var_name.into(), err))
    }
}

impl<T> IntoStoreError<T> for Result<T, StoreError> {
    fn with_store_error<S: Into<String>>(self: Self, user_name: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::NoHome(user_name.into(), Box::new(err)))
    }
}

impl<T> IntoStoreError<T> for Result<T, gpgme::Error> {
    fn with_store_error<S: Into<String>>(self: Self, op: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::Gpg(op.into(), err))
    }
}

impl<T> IntoStoreError<T> for Result<T, git2::Error> {
    fn with_store_error<S: Into<String>>(self: Self, operation: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::GitError(operation.into(), err))
    }
}

impl<T> IntoStoreError<T> for Result<T, PersistError> {
    fn with_store_error<S: Into<String>>(self: Self, passphrase_path: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::PassphrasePersist(passphrase_path.into(), err))
    }
}

#[cfg(feature = "parsed-passwords")]
impl<T> IntoStoreError<T> for Result<T, pest::error::Error<crate::parsed::Rule>> {
    fn with_store_error<S: Into<String>>(self: Self, path: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::Parse(path.into(), Box::new(err)))
    }
}
