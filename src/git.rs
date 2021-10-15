use std::path::Path;
use std::fmt;

use crate::DecryptedPassword;
use thiserror::Error;

use git2::{Config, Repository};
use custom_debug::Debug;

pub struct GitStatus;

fn debug_repository(repo: &Repository, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Repository {{ workdir: {:?}, state: {:?} }}", repo.workdir(), repo.state())
}

#[derive(Debug)]
pub struct Git {
    #[debug(with = "debug_repository")]
    repo: Repository,
    //config: Config,
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Could not open git repository")]
    Open(#[source] git2::Error),
}

type GitResult<T> = Result<T, GitError>;

impl Git {
    pub(crate) fn new(path: &Path) -> GitResult<Self> {
        let repo = Repository::open(path).map_err(|e| GitError::Open(e))?;

        Ok(Self { repo })
    }

    // TODO: handle merge conflict
    //       show user both decrypted passwords and let the user choose which to take
    pub fn pull<H>(&mut self, _merge_handler: H) -> GitResult<()>
    where
        H: Fn(&DecryptedPassword, &DecryptedPassword) -> DecryptedPassword,
    {
        Ok(())
    }

    pub fn push(&mut self) -> GitResult<()> {
        Ok(())
    }

    pub fn status(&mut self) -> GitResult<GitStatus> {
        Ok(GitStatus)
    }

    pub fn commit(&mut self, _message: &str) -> GitResult<()> {
        Ok(())
    }

    pub fn add(&mut self, _paths: &[&Path]) -> GitResult<()> {
        Ok(())
    }

    pub fn config_valid(&self) -> bool {
        let config = match Config::open_default() {
            Ok(config) => config,
            Err(_) => return false,
        };
        let mut entries = &match config.entries(None) {
            Ok(entries) => entries,
            Err(_) => return false,
        };

        let has_email = entries.any(|e| {
            let e = match e.ok() {
                Some(e) => e,
                None => return false,
            };
            let name = match e.name() {
                Some(name) => name,
                None => return false,
            };

            name == "user.email"
        });
        let has_name = entries.any(|e| {
            let e = match e.ok() {
                Some(e) => e,
                None => return false,
            };
            let name = match e.name() {
                Some(name) => name,
                None => return false,
            };

            name == "user.name"
        });

        has_email && has_name
    }

    pub fn config_set_user_name(&mut self, _name: &str) -> GitResult<()> {
        Ok(())
    }

    pub fn config_user_name(&mut self) -> GitResult<Option<String>> {
        Ok(None)
    }

    pub fn config_set_user_email(&mut self, _email: &str) -> GitResult<()> {
        Ok(())
    }

    pub fn config_user_email(&mut self) -> GitResult<Option<String>> {
        Ok(None)
    }
}
