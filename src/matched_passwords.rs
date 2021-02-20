use crate::{Entries, Password};

pub struct MatchedPasswords<'a, 'b> {
    pattern: &'b str,
    traverser: Entries<'a>,
}

impl<'a, 'b> MatchedPasswords<'a, 'b> {
    pub(crate) fn new(pattern: &'b str, traverser: Entries<'a>) -> Self {
        Self { pattern, traverser }
    }
}

impl<'a, 'b> Iterator for MatchedPasswords<'a, 'b> {
    type Item = Password;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.traverser.next() {
            if let Some(pass) = next.password() {
                if let Ok(decrypted) = pass.decrypt() {
                    if decrypted
                        .passphrase()
                        .to_lowercase()
                        .find(&self.pattern.to_lowercase())
                        .is_some()
                        || decrypted.comments().any(|(_pos, c)| {
                            c.to_lowercase()
                                .find(&self.pattern.to_lowercase())
                                .is_some()
                        })
                        || decrypted.all_entries().any(|(k, (_pos, v))| {
                            k.to_lowercase()
                                .find(&self.pattern.to_lowercase())
                                .is_some()
                                || v.to_lowercase()
                                    .find(&self.pattern.to_lowercase())
                                    .is_some()
                        })
                    {
                        return Some(pass);
                    }
                }
            }
        }

        None
    }
}
