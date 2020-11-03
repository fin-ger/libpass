use anyhow::Result;
use gpgme::{Context, Protocol};

use std::path::Path;
use std::fs::File;

pub struct DecryptedPassword {
    content: String,
}

impl DecryptedPassword {
    fn new(path: &Path) -> Result<Self> {
        let mut pw = File::open(path)?;
        let mut ctx = Context::from_protocol(Protocol::OpenPgp)?;
        let mut content = Vec::new();
        // TODO: Add password provider
        ctx.decrypt(&mut pw, &mut content)?;
        Ok(Self {
            content: String::from_utf8_lossy(&content).to_string(),
        })
    }

    pub fn content<'a>(&'a self) -> &'a str {
        &self.content
    }
}

pub struct Password<'a> {
    name: &'a str,
    path: &'a Path,
}

impl<'a> Password<'a> {
    pub(crate) fn new(name: &'a str, path: &'a Path) -> Self {
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

    pub fn decrypt(&self) -> Result<DecryptedPassword> {
        DecryptedPassword::new(self.path)
    }
}

