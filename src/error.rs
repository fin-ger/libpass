use std::{env, io, path::PathBuf};

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
    #[error("Could not open or modify entries in password store")]
    Io(String, #[source] io::Error),
    #[error("Password store path is not a directory: {0}")]
    NoDirectory(PathBuf),
    #[error("environment variable {0} is not set")]
    EnvVar(String, #[source] env::VarError),
    #[error("Cannot find home directory for current user: {0}")]
    NoHome(String, #[source] Box<StoreError>),
    #[error("Given path is not contained in the password store: {0}")]
    NotInStore(PathBuf),
    #[error("Failed to decrypt password {0}")]
    Decrypt(String, #[source] gpgme::Error),
    #[error("Failed to parse password content for {0}")]
    Parse(String, #[source] Box<dyn std::error::Error>),
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
    fn with_store_error<S: Into<String>>(self: Self, path: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::Decrypt(path.into(), err))
    }
}

#[cfg(feature = "parsed-passwords")]
impl<T> IntoStoreError<T> for Result<T, pest::error::Error<crate::parsed::Rule>> {
    fn with_store_error<S: Into<String>>(self: Self, path: S) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::Parse(path.into(), Box::new(err)))
    }
}
