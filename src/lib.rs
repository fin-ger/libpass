use id_tree::{Tree, Node, NodeId, InsertBehavior};
use anyhow::{anyhow, Context, Result, Error};
use directories::BaseDirs;
use bitflags::bitflags;

use std::{env, fs, slice};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

pub enum Location {
    /// $PASSWORD_STORE_DIR or if not set ~/.password-store
    Automatic,
    /// Override the path
    Manual(PathBuf),
}

enum PassNode {
    Directory {
        name: String,
        path: PathBuf,
    },
    Password {
        name: String,
        path: PathBuf,
    },
}

impl PassNode {
    fn is_dir(&self) -> bool {
        if let Self::Directory { .. } = self {
            true
        } else {
            false
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Directory { ref name, .. } => name,
            Self::Password { ref name, .. } => name,
        }
    }
}

bitflags! {
    pub struct Sorting: u8 {
        const NONE = 0;
        const ALPHABETICAL = 1;
        const DIRECTORIES_FIRST = 2;
    }
}

pub struct Store {
    path: PathBuf,
    tree: Tree<PassNode>,
}

impl Store {
    pub fn open(location: Location) -> Result<Self> {
        let path = match location {
            Location::Automatic => {
                env::var("PASSWORD_STORE_DIR")
                    .context("PASSWORD_STORE_DIR environment variable is not set")
                    .map(|password_store_dir| {
                        Path::new(&password_store_dir).to_owned()
                    })
                    .or_else(|e| {
                        BaseDirs::new()
                            .map(|base_dirs| base_dirs.home_dir().join(".password-store"))
                            .ok_or(e)
                            .context("Cannot find home directory for current user")
                    })?
            },
            Location::Manual(path) => path,
        };

        let tree = Tree::new();
        let mut me = Self {
            path,
            tree,
        };
        let _errors = me.load_passwords()?; // TODO: propagate errors

        Ok(me)
    }

    fn load_passwords(&mut self) -> Result<Vec<Error>> {
        self.tree = Tree::new();
        let root_id = self.tree.insert(
            Node::new(PassNode::Directory {
                name: ".".into(),
                path: self.path.clone(),
            }),
            InsertBehavior::AsRoot,
        ).expect("Failed to create internal tree representation");

        self.load_passwords_from_dir(&self.path.clone(), &root_id)
    }

    fn is_special_entry(path: &Path) -> bool {
        match path.file_name().unwrap_or("..".as_ref()).to_string_lossy().as_ref() {
            ".git" | ".gitattributes" | ".gpg-id" => true,
            _ => false,
        }
    }

    fn load_passwords_from_dir(&mut self, dir: &Path, parent: &NodeId) -> Result<Vec<Error>> {
        let (mut read_dir, mut errors): (Vec<_>, Vec<_>) = fs::read_dir(dir)
            .context(format!("Cannot open directory: {}", dir.display()))?
            .partition(Result::is_ok);

        let read_dir = read_dir.drain(..).map(|res| res.unwrap());
        let mut errors: Vec<_> = errors
            .drain(..)
            .map(|res| res.unwrap_err().into())
            .collect();

        for entry in read_dir {
            let path = entry.path();

            if path.is_dir() && !Self::is_special_entry(&path) {
                match path.file_name() {
                    Some(name) => {
                        let subdir = self.tree.insert(
                            Node::new(PassNode::Directory {
                                name: name.to_string_lossy().to_string(),
                                path: path.clone(),
                            }),
                            InsertBehavior::UnderNode(parent),
                        ).expect("Failed to insert directory into internal tree");
                        self.load_passwords_from_dir(&path, &subdir)?;
                    },
                    None => {
                        errors.push(anyhow!("Cannot get directory name for '{}'", path.display()));
                    },
                }
            } else if !Self::is_special_entry(&path) {
                match path.file_stem() {
                    Some(name) => {
                        let _pw_id = self.tree.insert(
                            Node::new(PassNode::Password {
                                name: name.to_string_lossy().to_string(),
                                path,
                            }),
                            InsertBehavior::UnderNode(parent),
                        ).expect("Failed to insert password into internal tree");
                    },
                    None => {
                        errors.push(anyhow!("Cannot get password name for '{}'", path.display()));
                    },
                }
            }
        }

        Ok(errors)
    }

    pub fn content(&self, sorting: Sorting) -> Directory {
        let root_id = self.tree.root_node_id()
            .expect("Failed to get root node of internal tree");
        Directory::new(".", &self.path, &self.tree, root_id, sorting)
    }
}

pub struct Directory<'a> {
    name: &'a str,
    path: &'a Path,
    tree: &'a Tree<PassNode>,
    entries: Vec<&'a NodeId>,
    sorting: Sorting,
}

impl<'a> Directory<'a> {
    fn new(
        name: &'a str,
        path: &'a Path,
        tree: &'a Tree<PassNode>,
        node: &NodeId,
        sorting: Sorting,
    ) -> Self {
        let mut entries: Vec<_> = tree.children_ids(node)
            .expect("Failed to read directory entries from internal tree")
            .collect();
        let sort_dirs = sorting.contains(Sorting::DIRECTORIES_FIRST);
        let sort_alpha = sorting.contains(Sorting::ALPHABETICAL);

        entries.sort_by(|a, b| {
            let a = tree.get(a).expect("Failed to find node in internal tree");
            let b = tree.get(b).expect("Failed to find node in internal tree");
            if sort_dirs && a.data().is_dir() && !b.data().is_dir() {
                Ordering::Less
            } else if sort_dirs && !a.data().is_dir() && b.data().is_dir() {
                Ordering::Greater
            } else if sort_alpha {
                let a_low = a.data().name().to_lowercase();
                let b_low = b.data().name().to_lowercase();

                a_low.cmp(&b_low)
            } else {
                Ordering::Less
            }
        });

        Self {
            name,
            path,
            tree,
            entries,
            sorting,
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn path(&self) -> &'a Path {
        self.path
    }

    pub fn passwords(&self) -> Passwords {
        Passwords {
            entries: self.entries(),
        }
    }

    pub fn directories(&self) -> Directories {
        Directories {
            entries: self.entries(),
        }
    }

    pub fn entries(&self) -> Entries {
        Entries {
            tree: self.tree,
            sorting: self.sorting.clone(),
            iter: self.entries.iter(),
        }
    }
}

pub struct DecryptedPassword {
}

impl DecryptedPassword {
    fn new(_path: &Path) -> Self {
        Self {
        }
    }
}

pub struct Password<'a> {
    name: &'a str,
    path: &'a Path,
}

impl<'a> Password<'a> {
    fn new(name: &'a str, path: &'a Path) -> Self {
        Self {
            name,
            path,
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn path(&self) -> &'a Path {
        self.path
    }

    pub fn decrypt(&self) -> DecryptedPassword {
        DecryptedPassword::new(self.path)
    }
}

pub struct Passwords<'a> {
    entries: Entries<'a>,
}

impl<'a> Iterator for Passwords<'a> {
    type Item = Password<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries
            .next()?
            .password()
    }
}

pub struct Directories<'a> {
    entries: Entries<'a>,
}

impl<'a> Iterator for Directories<'a> {
    type Item = Directory<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries
            .next()?
            .directory()
    }
}

pub enum DirectoryEntry<'a> {
    Password(Password<'a>),
    Directory(Directory<'a>),
}

impl<'a> DirectoryEntry<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            DirectoryEntry::Password(pw) => pw.name(),
            DirectoryEntry::Directory(dir) => dir.name(),
        }
    }

    pub fn path(&self) -> &'a Path {
        match self {
            DirectoryEntry::Password(pw) => pw.path(),
            DirectoryEntry::Directory(dir) => dir.path(),
        }
    }

    pub fn is_dir(&self) -> bool {
        if let DirectoryEntry::Directory(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_password(&self) -> bool {
        !self.is_dir()
    }

    pub fn password(self) -> Option<Password<'a>> {
        if let DirectoryEntry::Password(pw) = self {
            Some(pw)
        } else {
            None
        }
    }

    pub fn directory(self) -> Option<Directory<'a>> {
        if let DirectoryEntry::Directory(dir) = self {
            Some(dir)
        } else {
            None
        }
    }
}

pub struct Entries<'a> {
    tree: &'a Tree<PassNode>,
    sorting: Sorting,
    iter: slice::Iter<'a, &'a NodeId>,
}

impl<'a> Iterator for Entries<'a> {
    type Item = DirectoryEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.iter.next()?;
        let node = self.tree.get(id).expect("Failed to find node id in internal tree");

        Some(match node.data() {
            PassNode::Password { name, path } => {
                DirectoryEntry::Password(Password::new(name, path))
            },
            PassNode::Directory { name, path } => {
                DirectoryEntry::Directory(Directory::new(name, path, self.tree, id, self.sorting.clone()))
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use gpgme::{Context, KeyListMode, Protocol};
    use anyhow::Result;
    use crate::{Store, Location, Sorting, Directory};

    fn print_dir(dir: &Directory<'_>) {
        println!("Passwords:");
        for password in dir.passwords() {
            println!("  {}: {}", password.path().display(), password.name());
        }

        println!("Directories:");
        for dir in dir.directories() {
            println!("  {}: {}", dir.path().display(), dir.name());
        }

        println!("Entries:");
        for entry in dir.entries() {
            let kind = if entry.is_password() { "PW" } else { "DIR" };
            println!("  {} {}: {}", kind, entry.path().display(), entry.name());
        }

        println!();

        for dir in dir.directories() {
            print_dir(&dir);
        }
    }

    #[test]
    fn smoke() -> Result<()> {
        let store = Store::open(Location::Automatic)?;
        let content = store.content(Sorting::ALPHABETICAL | Sorting::DIRECTORIES_FIRST);
        print_dir(&content);
        Ok(())
    }

    #[test]
    fn run() -> Result<()> {
        let mode = KeyListMode::empty();
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
        ctx.set_key_list_mode(mode)?;
        let mut keys = ctx.secret_keys()?;
        for key in keys.by_ref().filter_map(|x| x.ok()) {
            println!("keyid   : {}", key.id().unwrap_or("?"));
            println!("fpr     : {}", key.fingerprint().unwrap_or("?"));
            println!(
                "caps    : {}{}{}{}",
                if key.can_encrypt() { "e" } else { "" },
                if key.can_sign() { "s" } else { "" },
                if key.can_certify() { "c" } else { "" },
                if key.can_authenticate() { "a" } else { "" }
            );
            println!(
                "flags   :{}{}{}{}{}{}",
                if key.has_secret() { " secret" } else { "" },
                if key.is_revoked() { " revoked" } else { "" },
                if key.is_expired() { " expired" } else { "" },
                if key.is_disabled() { " disabled" } else { "" },
                if key.is_invalid() { " invalid" } else { "" },
                if key.is_qualified() { " qualified" } else { "" }
            );
            for (i, user) in key.user_ids().enumerate() {
                println!("userid {}: {}", i, user.id().unwrap_or("[none]"));
                println!("valid  {}: {:?}", i, user.validity())
            }
            println!("");
        }

        Ok(())
    }
}
