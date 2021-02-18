#[derive(Debug, Clone)]
pub enum Umask {
    Automatic,
    Manual(u32),
}

impl<I> From<I> for Umask
where
    I: Into<u32>,
{
    fn from(mask: I) -> Umask {
        Umask::Manual(mask.into())
    }
}
