use std::path::Path;

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

    pub fn decrypt(&self) -> DecryptedPassword {
        DecryptedPassword::new(self.path)
    }
}

