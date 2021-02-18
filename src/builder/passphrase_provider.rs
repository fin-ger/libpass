use std::{
    fmt::{Debug, Formatter},
    io::Write,
    sync::Arc,
};

use gpgme::PassphraseRequest;

#[derive(Clone)]
pub enum PassphraseProvider {
    SystemAgent,
    Manual(Arc<dyn FnMut(PassphraseRequest, &mut dyn Write) -> Result<(), gpgme::Error>>),
}

impl Debug for PassphraseProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PassphraseProvider::SystemAgent => {
                let mut debug_trait_builder = f.debug_tuple("SystemAgent");
                debug_trait_builder.finish()
            }
            PassphraseProvider::Manual(_) => {
                let mut debug_trait_builder = f.debug_tuple("Manual");
                debug_trait_builder.field(&String::from(
                    "(PassphraseRequest, &mut dyn Write) -> Result<(), gpgme::Error>",
                ));
                debug_trait_builder.finish()
            }
        }
    }
}

impl<F> From<F> for PassphraseProvider
where
    F: FnMut(PassphraseRequest, &mut dyn Write) -> Result<(), gpgme::Error> + 'static,
{
    fn from(func: F) -> PassphraseProvider {
        PassphraseProvider::Manual(Arc::new(func))
    }
}
