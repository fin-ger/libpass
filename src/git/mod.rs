mod conflict_resolver;
mod conflicted_password;
mod conflicted_gpg_id;
mod conflicted_plain_text;
mod conflicted_binary;

use std::{fmt, path::PathBuf};
use std::path::Path;

pub use conflict_resolver::ConflictResolver;

use custom_debug::Debug;
use git2::{AnnotatedCommit, AutotagOption, Config, ErrorClass, ErrorCode, FetchOptions, IndexAddOption, ObjectType, Reference, Repository, StatusOptions, build::CheckoutBuilder};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct GitStatusEntry {
    path: PathBuf,
    status: GitStatus,
}

impl GitStatusEntry {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn status(&self) -> GitStatus {
        self.status
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GitStatus {
    New,
    Modified,
    Deleted,
    Renamed,
    Typechange,
}

#[derive(Debug, Clone)]
pub struct GitStatuses {
    pub staging: Vec<GitStatusEntry>,
    pub workdir: Vec<GitStatusEntry>,
    pub conflicts: Vec<PathBuf>,
}

impl GitStatuses {
    fn new_from(statuses: git2::Statuses) -> Self {
        let mut staging = Vec::new();
        let mut workdir = Vec::new();
        let mut conflicts = Vec::new();

        for status_entry in statuses.iter() {
            let path = Path::new(status_entry.path().expect("Filename not valid utf-8 🤷")).to_owned();
            let status = status_entry.status();

            if status.is_conflicted() {
                conflicts.push(path);
            } else if status.is_index_deleted() {
                staging.push(GitStatusEntry { status: GitStatus::Deleted, path });
            } else if status.is_index_modified() {
                staging.push(GitStatusEntry { status: GitStatus::Modified, path });
            } else if status.is_index_new() {
                staging.push(GitStatusEntry { status: GitStatus::New, path });
            } else if status.is_index_renamed() {
                staging.push(GitStatusEntry { status: GitStatus::Renamed, path });
            } else if status.is_index_typechange() {
                staging.push(GitStatusEntry { status: GitStatus::Typechange, path });
            } else if status.is_wt_deleted() {
                workdir.push(GitStatusEntry { status: GitStatus::Deleted, path });
            } else if status.is_wt_modified() {
                workdir.push(GitStatusEntry { status: GitStatus::Modified, path });
            } else if status.is_wt_new() {
                workdir.push(GitStatusEntry { status: GitStatus::New, path });
            } else if status.is_wt_renamed() {
                workdir.push(GitStatusEntry { status: GitStatus::Renamed, path });
            } else if status.is_wt_typechange() {
                workdir.push(GitStatusEntry { status: GitStatus::Typechange, path });
            } else {
                // this should never be reached as we opted out for ignored files
                unreachable!();
            }
        }

        Self {
            staging,
            workdir,
            conflicts,
        }
    }

    pub fn is_clean(&self) -> bool {
        self.staging.is_empty() && self.workdir.is_empty() && self.conflicts.is_empty()
    }
}

fn debug_repository(repo: &Repository, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
        f,
        "Repository {{ workdir: {:?}, state: {:?} }}",
        repo.workdir(),
        repo.state()
    )
}

#[derive(Debug)]
pub struct Git {
    #[debug(with = "debug_repository")]
    repo: Repository,
}

#[derive(Debug)]
pub enum GitRemote {
    UpstreamForBranch,
    Manual(String),
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

    pub fn fetch(&self) -> GitResult<()> {
        for remote in self.repo.remotes()?.into_iter() {
            let mut remote = self
                .repo
                .find_remote(remote.expect("Remote name not valid utf-8 🤷"))?;
            let mut fo = FetchOptions::new();
            fo.download_tags(AutotagOption::All);
            remote.fetch::<&str>(&[], Some(&mut fo), None)?;
        }

        Ok(())
    }

    fn fast_forward<'a>(
        &'a self,
        local_branch_name: String,
        mut local_branch_ref: Reference<'a>,
        remote_commit: AnnotatedCommit<'a>,
    ) -> Result<ConflictResolver<'a>, git2::Error> {
        Ok(ConflictResolver::new_without_conflicts(&self.repo, move |repo, _idx| {
            local_branch_ref.set_target(
                remote_commit.id(),
                &format!(
                    "Fast-Forward: Setting {} to id: {}",
                    local_branch_name,
                    remote_commit.id()
                ),
            )?;
            repo.set_head(&local_branch_name)?;
            repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
            Ok(())

        }))
    }

    fn set_head<'a>(
        &'a self,
        local_branch_name: String,
        remote_commit: AnnotatedCommit<'a>,
    ) -> Result<ConflictResolver<'a>, git2::Error> {
        Ok(ConflictResolver::new_without_conflicts(&self.repo, move |repo, _idx| {
            repo.reference(
                &local_branch_name,
                remote_commit.id(),
                true,
                &format!(
                    "Setting {} to {}",
                    local_branch_name,
                    remote_commit.id()
                ),
            )?;
            repo.set_head(&local_branch_name)?;
            repo.checkout_head(Some(
                CheckoutBuilder::default()
                    .allow_conflicts(true)
                    .conflict_style_merge(true)
                    .force(),
            ))?;

            Ok(())
        }))
    }

    fn normal_merge<'a>(
        &'a self,
        local_commit_name: String,
        local_commit: AnnotatedCommit<'a>,
        remote_commit_name: String,
        remote_commit: AnnotatedCommit<'a>,
    ) -> GitResult<ConflictResolver<'a>> {
        let local_tree = self.repo.find_commit(local_commit.id())?.tree()?;
        let remote_tree = self.repo.find_commit(remote_commit.id())?.tree()?;
        let ancestor = self.repo
            .find_commit(self.repo.merge_base(local_commit.id(), remote_commit.id())?)?
            .tree()?;
        let idx = self.repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

        ConflictResolver::from_index(idx, &self.repo, move |repo, idx| {
            let mut idx = idx.unwrap();
            if idx.has_conflicts() {
                return Err(git2::Error::new(git2::ErrorCode::Conflict, git2::ErrorClass::Merge, "Not all conflicts resolved"));
            }
            let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
            // now create the merge commit
            let msg = format!("Merge {} into {}", remote_commit_name.trim_start_matches("refs/remotes/"), local_commit_name.trim_start_matches("refs/heads/"));
            let sig = repo.signature()?;
            let local_commit = repo.find_commit(local_commit.id())?;
            let remote_commit = repo.find_commit(remote_commit.id())?;
            // Do our merge commit and set current branch head to that commit.
            let _merge_commit = repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &msg,
                &result_tree,
                &[&local_commit, &remote_commit],
            )?;

            // Set working tree to match head.
            repo.checkout_head(Some(CheckoutBuilder::default().force()))?;

            Ok(())
        })
    }

    pub fn merge(&mut self) -> GitResult<ConflictResolver> {
        if !self.status()?.is_clean() {
            return Err(git2::Error::new(ErrorCode::Modified, ErrorClass::Merge, "Repository status is not clean"));
        }

        let head = self.repo.head()?;
        let current_branch_name = head.name().expect("Branch name not valid utf-8 🤷");
        let current_branch_ref = self
            .repo
            .find_reference(current_branch_name)?;
        let current_branch = self
            .repo
            .reference_to_annotated_commit(&current_branch_ref)?;
        let upstream_branch_name = self
            .repo
            .branch_upstream_name(current_branch_name)?
            .as_str()
            .expect("Remote branch name not valid utf-8 🤷")
            .to_owned();
        let upstream_branch_ref = self.repo.find_reference(&upstream_branch_name)?;
        let upstream_branch = self
            .repo
            .reference_to_annotated_commit(&upstream_branch_ref)?;

        let analysis = self.repo.merge_analysis_for_ref(&current_branch_ref, &[&upstream_branch])?;
        if analysis.0.is_none() {
            return Err(git2::Error::new(
                ErrorCode::Unmerged,
                ErrorClass::Merge,
                "Merge is not possible",
            ));
        } else if analysis.0.is_up_to_date() {
            Ok(ConflictResolver::new_without_conflicts(&self.repo, |_repo, _idx| {
                Ok(()) // do nothing
            }))
        } else if analysis.0.is_fast_forward() {
            self.fast_forward(
                current_branch_name.to_owned(),
                current_branch_ref,
                upstream_branch,
            )
        } else if analysis.0.is_normal() {
            self.normal_merge(current_branch_name.to_owned(), current_branch, upstream_branch_name.to_owned(), upstream_branch)
        } else if analysis.0.is_unborn() {
            self.set_head(current_branch_name.to_owned(), upstream_branch)
        } else {
            unreachable!();
        }
    }

    pub fn pull(&mut self) -> GitResult<ConflictResolver> {
        self.fetch()?;
        self.merge()
    }

    pub fn push(&mut self, remote: GitRemote) -> GitResult<()> {
        let head = self.repo.head()?;
        let branch_name = head.name().expect("Branch name not valid utf-8 🤷");
        let remote = match remote {
            GitRemote::UpstreamForBranch => self
                .repo
                .branch_upstream_remote(branch_name)?
                .as_str()
                .expect("Remote name not valid utf-8 🤷")
                .to_owned(),
            GitRemote::Manual(remote) => remote,
        };

        let mut remote = self.repo.find_remote(&remote)?;
        remote.push(&[branch_name], None)?;

        Ok(())
    }

    pub fn status(&mut self) -> GitResult<GitStatuses> {
        let mut opts = StatusOptions::new();
        opts.include_ignored(false);
        opts.include_untracked(true);
        opts.exclude_submodules(true);
        let statuses = self.repo.statuses(Some(&mut opts))?;

        Ok(GitStatuses::new_from(statuses))
    }

    pub(crate) fn commit<M: Into<String>>(&mut self, message: M) -> GitResult<()> {
        let me = self.repo.signature()?;
        let tree_id = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        // FIXME: closure hack until try-blocks are stable
        let last_commit = (|| {
            self.repo
                .head()
                .ok()?
                .resolve()
                .ok()?
                .peel(ObjectType::Commit)
                .ok()?
                .into_commit()
                .ok()
        })();
        let parents = last_commit.iter().collect::<Vec<_>>();

        self.repo
            .commit(Some("HEAD"), &me, &me, &message.into(), &tree, &parents)?;
        Ok(())
    }

    pub(crate) fn add(&mut self, paths: &[&Path]) -> GitResult<()> {
        let workdir = self.repo.workdir().unwrap();
        for path in paths {
            let relative = path.strip_prefix(workdir).unwrap();
            if path.exists() {
                if path.is_dir() {
                    for file in WalkDir::new(path)
                        .follow_links(true)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        let filename = file.file_name().to_string_lossy();
                        let relative = file.path().strip_prefix(workdir).unwrap();

                        if filename.ends_with(".gpg") || filename == ".gpg-id" {
                            // forcefully add passwords and gpg-id files
                            self.repo.index()?.add_path(relative)?;
                        } else {
                            // other files are checked against gitignore and such
                            self.repo.index()?.add_all(
                                &[relative],
                                IndexAddOption::DEFAULT,
                                None,
                            )?;
                        }
                    }
                } else {
                    self.repo.index()?.add_path(relative)?;
                }
            } else {
                self.repo.index()?.remove_all(&[relative], None)?;
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
