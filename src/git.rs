use std::path::Path;
use std::fmt;

use crate::DecryptedPassword;

use git2::{Config, ObjectType, Repository};
use custom_debug::Debug;

pub struct GitStatus;

fn debug_repository(repo: &Repository, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Repository {{ workdir: {:?}, state: {:?} }}", repo.workdir(), repo.state())
}

#[derive(Debug)]
pub struct Git {
    #[debug(with = "debug_repository")]
    repo: Repository,
}

type GitResult<T> = Result<T, git2::Error>;

impl Git {
    pub(crate) fn open(path: &Path) -> GitResult<Option<Self>> {
        if path.join(".git").is_dir() {
            let repo = Repository::open(path)?;
            Ok(Some(Self { repo }))
        } else {
            Ok(None)
        }
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

    pub(crate) fn commit(&mut self, message: &str) -> GitResult<()> {
        let me = self.repo.signature()?;
        let tree_id = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        // FIXME: closure hack until try-blocks are stable
        let last_commit = (|| {
            self.repo
                .head().ok()?
                .resolve().ok()?
                .peel(ObjectType::Commit).ok()?
                .into_commit().ok()
        })();
        let parents = last_commit.iter().collect::<Vec<_>>();

        self.repo.commit(
            Some("HEAD"),
            &me,
            &me,
            message,
            &tree,
            &parents,
        )?;
        Ok(())
    }

    pub(crate) fn add(&mut self, paths: &[&Path]) -> GitResult<()> {
        let workdir = self.repo.workdir().unwrap();
        for path in paths {
            let relative = path.strip_prefix(workdir).unwrap();
            if path.exists() {
                self.repo.index()?.add_path(relative)?;
            } else {
                self.repo.index()?.remove_path(relative)?;
            }
        }
        self.repo.index()?.write()?;
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
