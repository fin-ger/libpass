use std::cmp::Ordering;
use std::path::PathBuf;
use std::{env, fs, path::Path};

use directories::BaseDirs;
use id_tree::{InsertBehavior, Node, NodeId, Tree};

use crate::{
    DecryptedPassword, Directory, DirectoryInserter, Entries, Entry, Git, IntoStoreError, Location,
    MatchedEntries, MatchedPasswords, MutDirectory, MutEntry, MutPassword, PassNode,
    PassphraseProvider, Password, PasswordInserter, SigningKey, Sorting, StoreError, StoreErrors,
    TraversalOrder, Umask,
};

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    pub(crate) tree: Tree<PassNode>,
    errors: Vec<StoreError>,
    git: Option<Git>,
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
        todo!();
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
                .with_store_error("PASSWORD_STORE_DIR")
                .map(|password_store_dir| Path::new(&password_store_dir).to_owned())
                .or_else(|e| {
                    BaseDirs::new()
                        .map(|base_dirs| base_dirs.home_dir().join(".password-store"))
                        .ok_or(e)
                        .with_store_error("attempted search in default paths")
                })?,
            Location::Manual(path) => path,
        };
        let path = path
            .canonicalize()
            .with_store_error(path.display().to_string())?;
        let metadata = path
            .metadata()
            .with_store_error(path.display().to_string())?;
        if !metadata.is_dir() {
            return Err(StoreError::NoDirectory(path));
        }

        let tree = Tree::new();
        let git = Git::open(&path).with_store_error("open repository")?;
        let mut me = Self {
            path,
            tree,
            git,
            errors: Vec::new(),
        };
        me.load_passwords();
        me.sort(sorting);

        Ok(me)
    }

    pub(crate) fn tree(&self) -> &Tree<PassNode> {
        &self.tree
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

    fn is_password(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            ext == "gpg"
        } else {
            false
        }
    }

    fn load_passwords_from_dir(&mut self, dir: &Path, parent: &NodeId) {
        let (mut read_dir, errors) = match fs::read_dir(dir).with_store_error(dir.display().to_string())
        {
            Ok(read_dir) => {
                let (mut read_dir_res, mut errors_res): (Vec<_>, Vec<_>) =
                    read_dir.partition(Result::is_ok);
                (
                    read_dir_res.drain(..).map(|res| res.unwrap()).collect(),
                    errors_res
                        .drain(..)
                        .map(|res| res.with_store_error(dir.display().to_string()).unwrap_err())
                        .collect(),
                )
            }
            Err(err) => (vec![], vec![err]),
        };

        read_dir.sort_by(|a, b| {
            a.path().cmp(&b.path())
        });
        read_dir.sort_by(|a, b| {
            let mut a_is_file = true;
            let mut b_is_file = true;
            if let Ok(file_type) = a.file_type() {
                a_is_file = file_type.is_file();
            }
            if let Ok(file_type) = b.file_type() {
                b_is_file = file_type.is_file();
            }

            if a_is_file && !b_is_file {
                Ordering::Greater
            } else if !a_is_file && b_is_file {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });

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
                        // this only triggers when the path is ".." and can therefore be ignored
                    }
                }
            } else if !Self::is_special_entry(&path) && Self::is_password(&path) {
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
                        // this only triggers when the path is "..":
                        //   if the filename has no stem (no dot) then the whole filename is used
                        //   if the filename starts with a dot then the whole filename is used
                        //  therefore this can be ignored.
                    }
                }
            } else if !Self::is_special_entry(&path) && !Self::is_password(&path) {
                let _f_id = self
                    .tree
                    .insert(
                        Node::new(PassNode::NormalFile {
                            name: path.file_name().expect("Path terminated with ...! Looks like your directory structure is too deep. If you are willing to implement a fix, please don't hesitate to do so. I have no interest in fixing this. To workaround this problem just use less folders.").to_string_lossy().to_string(),
                            path,
                        }),
                        InsertBehavior::UnderNode(parent),
                    )
                    .expect("Failed to insert normal file into internal tree");
            }
        }

        self.errors.extend(errors);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> StoreErrors {
        StoreErrors::new(&self.errors)
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

    pub fn has_git(&self) -> bool {
        self.git.is_some()
    }

    pub fn git<'a>(&'a mut self) -> Option<&'a mut Git> {
        self.git.as_mut()
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
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                if let Ok(path) = path.with_extension("gpg").canonicalize() {
                    path
                } else {
                    return Err(err).with_store_error(path.display().to_string());
                }
            }
            Err(err) => {
                return Err(err).with_store_error(path.display().to_string());
            }
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
                return Ok(Entries::new(&self.tree, self.path.clone(), node_id, order));
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

        Ok(Entries::new(&self.tree, self.path.clone(), node_id, order))
    }

    pub fn mut_directory(&mut self, directory: Directory) -> MutDirectory {
        MutDirectory::new(directory.node_id().to_owned(), self)
    }

    pub fn mut_password(&mut self, password: Password) -> MutPassword {
        MutPassword::new(password.node_id().to_owned(), self)
    }

    pub fn mut_entry(&mut self, entry: Entry) -> MutEntry {
        MutEntry::new(entry.node_id().to_owned(), self)
    }

    fn insert_password_into_tree(
        &mut self,
        name: String,
        path: PathBuf,
        parent: &NodeId,
    ) -> Password {
        let node = Node::new(PassNode::Password {
            name: name.clone(),
            path: path.clone(),
        });
        let node_id = self
            .tree
            .insert(node, InsertBehavior::UnderNode(parent))
            .expect("Parent of inserted password does not exist in internal tree");

        Password::new(name, path, self.path.clone(), node_id)
    }

    fn insert_directory_into_tree(
        &mut self,
        name: String,
        path: PathBuf,
        parent: &NodeId,
    ) -> Directory {
        let node = Node::new(PassNode::Directory {
            name: name.clone(),
            path: path.clone(),
        });
        let node_id = self
            .tree
            .insert(node, InsertBehavior::UnderNode(parent))
            .expect("Parent of inserted directory does not exist in internal tree");

        Directory::new(name, path, self.path.clone(), node_id)
    }

    pub fn insert_password(&mut self, inserter: &PasswordInserter) -> Result<Password, StoreError> {
        DecryptedPassword::create_and_write(
            inserter.lines.clone(),
            &self.path.join(&inserter.path),
            inserter.changes.clone(),
            self,
        )?;

        Ok(self.insert_password_into_tree(
            inserter.name.clone(),
            inserter.path.clone(),
            &inserter.parent,
        ))
    }

    #[cfg(feature = "parsed-passwords")]
    pub fn insert_parsed_password(
        &mut self,
        inserter: &crate::parsed::PasswordInserter,
    ) -> Result<Password, StoreError> {
        crate::parsed::DecryptedPassword::create_and_write(
            inserter.passphrase.clone(),
            inserter.lines.clone(),
            &inserter.path,
            inserter.changes.clone(),
            self,
        )?;

        Ok(self.insert_password_into_tree(
            inserter.name.clone(),
            inserter.path.clone(),
            &inserter.parent,
        ))
    }

    pub fn insert_directory(
        &mut self,
        inserter: &DirectoryInserter,
    ) -> Result<Directory, StoreError> {
        fs::create_dir(&inserter.path).with_store_error(inserter.path.display().to_string())?;

        Ok(self.insert_directory_into_tree(
            inserter.name.clone(),
            inserter.path.clone(),
            &inserter.parent,
        ))
    }

    pub fn location(&self) -> &Path {
        &self.path
    }
}
