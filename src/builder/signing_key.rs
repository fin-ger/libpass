#[derive(Debug, Clone)]
pub enum SigningKey {
    Automatic,
    Manual(String),
}

impl<K> From<K> for SigningKey
where
    K: Into<String>,
{
    fn from(signing_key: K) -> SigningKey {
        SigningKey::Manual(signing_key.into())
    }
}
