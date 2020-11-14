use std::path::{Path, PathBuf};

use thiserror::Error;
use crate::DecryptedPassword;

pub struct GitStatus;

pub struct GitRepository {
    path: PathBuf,
}

#[derive(Error, Debug)]
pub enum GitError {
}

type GitResult<T> = Result<T, GitError>;

impl GitRepository {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path
        }
    }

    // TODO: handle merge conflict
    //       show user both decrypted passwords and let the user choose which to take
    pub fn pull<H>(&mut self, merge_handler: H) -> GitResult<()>
    where H: Fn(&DecryptedPassword, &DecryptedPassword) -> DecryptedPassword {
        Ok(())
    }

    pub fn push(&mut self) -> GitResult<()> {
        Ok(())
    }

    pub fn status(&mut self) -> GitResult<GitStatus> {
        Ok(GitStatus)
    }

    pub fn commit(&mut self, message: &str) -> GitResult<()> {
        Ok(())
    }

    pub fn add(&mut self, paths: &[&Path]) -> GitResult<()> {
        Ok(())
    }

    pub fn config_valid(&mut self) -> bool {
        false
    }

    pub fn config_set_user_name(&mut self, name: &str) -> GitResult<()> {
        Ok(())
    }

    pub fn config_user_name(&mut self) -> GitResult<Option<String>> {
        Ok(None)
    }

    pub fn config_set_user_email(&mut self, email: &str) -> GitResult<()> {
        Ok(())
    }

    pub fn config_user_email(&mut self) -> GitResult<Option<String>> {
        Ok(None)
    }
}
