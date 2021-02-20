use std::path::PathBuf;
use std::{env, fs, io, path::Path};

use directories::BaseDirs;
use id_tree::{InsertBehavior, Node, NodeId, Tree};
use thiserror::Error;

use crate::{
    Directory, Entries, Entry, IntoStoreReadError, Location, MatchedEntries, MatchedPasswords,
    MutDirectory, MutEntry, MutPassword, PassNode, PassphraseProvider, Password, SigningKey,
    Sorting, StoreReadError, TraversalOrder, Umask,
};

pub struct StoreReadErrors<'a> {
    iter: std::slice::Iter<'a, StoreReadError>,
}

impl<'a> StoreReadErrors<'a> {
    fn new(errors: &'a Vec<StoreReadError>) -> Self {
        Self {
            iter: errors.into_iter(),
        }
    }
}

impl<'a> Iterator for StoreReadErrors<'a> {
    type Item = &'a StoreReadError;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Could not open or modify entries in password store")]
    Io(#[source] io::Error),
    #[error("Password store path is not a directory: {0}")]
    NoDirectory(PathBuf),
    #[error("PASSWORD_STORE_DIR environment variable is not set")]
    EnvVar(#[source] env::VarError),
    #[error("Cannot find home directory for current user")]
    NoHome(#[source] Box<StoreError>),
    #[error("Given path is not contained in the password store: {0}")]
    NotInStore(PathBuf),
    #[error("Given path does not exist: {0}")]
    DoesNotExist(#[source] io::Error),
}

pub(crate) trait IntoStoreError<T> {
    fn with_store_error(self: Self) -> Result<T, StoreError>;
}

impl<T> IntoStoreError<T> for Result<T, env::VarError> {
    fn with_store_error(self: Self) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::EnvVar(err))
    }
}

impl<T> IntoStoreError<T> for Result<T, StoreError> {
    fn with_store_error(self: Self) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::NoHome(Box::new(err)))
    }
}

impl<T> IntoStoreError<T> for Result<T, io::Error> {
    fn with_store_error(self: Self) -> Result<T, StoreError> {
        self.map_err(|err| StoreError::Io(err))
    }
}

pub struct Store {
    path: PathBuf,
    tree: Tree<PassNode>,
    errors: Vec<StoreReadError>,
}

impl Store {
    pub(crate) fn init(
        _location: Location,
        _passphrase_provider: PassphraseProvider,
        _umask: Umask,
        _signing_key: SigningKey,
        _sorting: Sorting,
        _key_id: &str,
    ) -> Result<Self, StoreError> {
        unimplemented!();
    }

    pub(crate) fn open(
        location: Location,
        _passphrase_provider: PassphraseProvider,
        _umask: Umask,
        _signing_key: SigningKey,
        sorting: Sorting,
    ) -> Result<Self, StoreError> {
        let path = match location {
            Location::Automatic => env::var("PASSWORD_STORE_DIR")
                .with_store_error()
                .map(|password_store_dir| Path::new(&password_store_dir).to_owned())
                .or_else(|e| {
                    BaseDirs::new()
                        .map(|base_dirs| base_dirs.home_dir().join(".password-store"))
                        .ok_or(e)
                        .with_store_error()
                })?,
            Location::Manual(path) => path,
        };
        let path = match path.canonicalize() {
            Ok(path) => path,
            Err(err) => return Err(StoreError::Io(err)),
        };

        let metadata = path.metadata().with_store_error()?;
        if !metadata.is_dir() {
            return Err(StoreError::NoDirectory(path));
        }

        let tree = Tree::new();
        let mut me = Self {
            path,
            tree,
            errors: Vec::new(),
        };
        me.load_passwords();
        me.sort(sorting);

        Ok(me)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> StoreReadErrors {
        StoreReadErrors::new(&self.errors)
    }

    pub fn sort(&mut self, sorting: Sorting) {
        if sorting.contains(Sorting::NONE) {
            return;
        }

        let root_id = self
            .tree
            .root_node_id()
            .expect("Cannot find root node in internal tree")
            .clone();
        let level_order = self
            .tree
            .traverse_level_order_ids(&root_id)
            .expect("Failed to traverse internal tree")
            .collect::<Vec<_>>();

        for node_id in level_order {
            self.tree
                .sort_children_by(&node_id, |a, b| sorting.cmp(a, b))
                .expect("Failed to sort internal tree");
        }
    }

    fn load_passwords(&mut self) {
        self.tree = Tree::new();
        let root_id = self
            .tree
            .insert(
                Node::new(PassNode::Directory {
                    name: ".".into(),
                    path: self.path.clone(),
                }),
                InsertBehavior::AsRoot,
            )
            .expect("Failed to create internal tree representation");

        self.load_passwords_from_dir(&self.path.clone(), &root_id);
    }

    fn is_special_entry(path: &Path) -> bool {
        match path
            .file_name()
            .unwrap_or("..".as_ref())
            .to_string_lossy()
            .as_ref()
        {
            ".git" | ".gitattributes" | ".gpg-id" => true,
            _ => false,
        }
    }

    fn load_passwords_from_dir(&mut self, dir: &Path, parent: &NodeId) {
        let (read_dir, mut errors) = match fs::read_dir(dir).with_store_read_error(dir) {
            Ok(read_dir) => {
                let (mut read_dir_res, mut errors_res): (Vec<_>, Vec<_>) =
                    read_dir.partition(Result::is_ok);
                (
                    read_dir_res.drain(..).map(|res| res.unwrap()).collect(),
                    errors_res
                        .drain(..)
                        .map(|res| res.with_store_read_error(dir).unwrap_err())
                        .collect(),
                )
            }
            Err(err) => (vec![], vec![err]),
        };

        for entry in read_dir {
            let path = entry.path();

            if path.is_dir() && !Self::is_special_entry(&path) {
                match path.file_name() {
                    Some(name) => {
                        let subdir = self
                            .tree
                            .insert(
                                Node::new(PassNode::Directory {
                                    name: name.to_string_lossy().to_string(),
                                    path: path.clone(),
                                }),
                                InsertBehavior::UnderNode(parent),
                            )
                            .expect("Failed to insert directory into internal tree");
                        self.load_passwords_from_dir(&path, &subdir);
                    }
                    None => {
                        errors.push(StoreReadError::name_error(&path));
                    }
                }
            } else if !Self::is_special_entry(&path) {
                match path.file_stem() {
                    Some(name) => {
                        let _pw_id = self
                            .tree
                            .insert(
                                Node::new(PassNode::Password {
                                    name: name.to_string_lossy().to_string(),
                                    path,
                                }),
                                InsertBehavior::UnderNode(parent),
                            )
                            .expect("Failed to insert password into internal tree");
                    }
                    None => {
                        errors.push(StoreReadError::name_error(&path));
                    }
                }
            }
        }

        self.errors.extend(errors);
    }

    pub fn find<'a, 'b>(&'a self, pattern: &'b str) -> MatchedEntries<'a, 'b> {
        MatchedEntries::new(
            pattern,
            self.show(".", TraversalOrder::PreOrder)
                .expect("Root node of internal tree could not be found"),
        )
    }

    pub fn grep<'a, 'b>(&'a self, pattern: &'b str) -> MatchedPasswords<'a, 'b> {
        MatchedPasswords::new(
            pattern,
            self.show(".", TraversalOrder::PostOrder)
                .expect("Root node of internal tree could not be found"),
        )
    }

    /// Either a relative path from the store's root or an absolute path where the
    /// password store's location is a prefix of the path.
    ///
    /// Example:
    ///
    /// "/path/to/password/store/and/password.gpg" for an absolute path where the password store is located at "/path/to/password/store"
    ///
    /// "./and/password.gpg" for a relative path inside the password store
    pub fn show<P: AsRef<Path>>(
        &self,
        path: P,
        order: TraversalOrder,
    ) -> Result<Entries, StoreError> {
        let mut path = path.as_ref().to_owned();
        if path.is_relative() {
            path = self.path.join(path);
        } else if path.strip_prefix(&self.path).is_err() {
            return Err(StoreError::NotInStore(path));
        }

        let path = match path.canonicalize() {
            Ok(path) => path,
            Err(err) => return Err(StoreError::DoesNotExist(err)),
        };

        let mut id = self.tree.root_node_id();
        if let Some(node_id) = id {
            let root_path = self
                .tree
                .get(node_id)
                .expect("Root node not available in internal tree")
                .data()
                .path();
            if path == root_path {
                return Ok(Entries::new(&self.tree, node_id, order));
            }
        }

        'search: while let Some(node_id) = id {
            id = None;
            let children_ids = self
                .tree
                .children_ids(node_id)
                .expect("Failed to get children of path node in internal tree");
            'children: for child_id in children_ids {
                let node_path = self
                    .tree
                    .get(child_id)
                    .expect("Failed to get data of node in internal tree")
                    .data()
                    .path();
                if path == node_path {
                    // complete path found
                    id = Some(child_id);
                    break 'search;
                } else if path.starts_with(node_path) {
                    // next level in path found
                    id = Some(child_id);
                    break 'children;
                }
            }
        }

        // This should never fail as path has been checked for canonicalization
        // which ensures the path exists on the filesystem.
        let node_id =
            id.expect("Store entry not found in password store although it must be available!");

        Ok(Entries::new(&self.tree, node_id, order))
    }

    pub fn mut_directory(&mut self, directory: Directory) -> MutDirectory {
        MutDirectory::new(
            directory.name().to_string(),
            directory.path().to_owned(),
            &mut self.tree,
            directory.node_id().to_owned(),
        )
    }

    pub fn mut_password(&mut self, password: Password) -> MutPassword {
        MutPassword::new(
            password.name().to_owned(),
            password.path().to_owned(),
            &mut self.tree,
            password.node_id().to_owned(),
        )
    }

    pub fn mut_entry(&mut self, entry: Entry) -> MutEntry {
        MutEntry::new(entry.node_id().to_owned(), &mut self.tree)
    }

    pub fn location(&self) -> &Path {
        &self.path
    }
}
