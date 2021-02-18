use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Location {
    /// $PASSWORD_STORE_DIR or if not set ~/.password-store
    Automatic,
    /// Override the path
    Manual(PathBuf),
}

impl<P> From<P> for Location
where
    P: Into<PathBuf>,
{
    fn from(path: P) -> Location {
        Location::Manual(path.into())
    }
}
