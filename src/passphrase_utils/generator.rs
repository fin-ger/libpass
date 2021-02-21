use passwords::PasswordGenerator;

use crate::StoreError;

pub struct PassphraseGenerator<'a> {
    generator: PasswordGenerator,
    password_handler: Box<dyn 'a + FnOnce(String) -> Result<(), StoreError>>,
}

impl<'a> PassphraseGenerator<'a> {
    pub(crate) fn new<F: 'a + FnOnce(String) -> Result<(), StoreError>>(
        password_handler: F,
    ) -> Self {
        Self {
            generator: PasswordGenerator::new(),
            password_handler: Box::new(password_handler),
        }
    }

    pub fn length(self, length: usize) -> Self {
        Self {
            generator: PasswordGenerator {
                length,
                ..self.generator
            },
            ..self
        }
    }

    pub fn numbers(self, numbers: bool) -> Self {
        Self {
            generator: PasswordGenerator {
                numbers,
                ..self.generator
            },
            ..self
        }
    }

    pub fn lowercase_letters(self, lowercase_letters: bool) -> Self {
        Self {
            generator: PasswordGenerator {
                lowercase_letters,
                ..self.generator
            },
            ..self
        }
    }

    pub fn uppercase_letters(self, uppercase_letters: bool) -> Self {
        Self {
            generator: PasswordGenerator {
                uppercase_letters,
                ..self.generator
            },
            ..self
        }
    }

    pub fn symbols(self, symbols: bool) -> Self {
        Self {
            generator: PasswordGenerator {
                symbols,
                ..self.generator
            },
            ..self
        }
    }

    pub fn spaces(self, spaces: bool) -> Self {
        Self {
            generator: PasswordGenerator {
                spaces,
                ..self.generator
            },
            ..self
        }
    }

    pub fn exclude_similar_characters(self, exclude_similar_characters: bool) -> Self {
        Self {
            generator: PasswordGenerator {
                exclude_similar_characters,
                ..self.generator
            },
            ..self
        }
    }

    pub fn strict(self, strict: bool) -> Self {
        Self {
            generator: PasswordGenerator {
                strict,
                ..self.generator
            },
            ..self
        }
    }

    pub fn generate(self, count: usize) -> Result<GeneratedPassphrases<'a>, StoreError> {
        let passwords = self
            .generator
            .generate(count)
            .map_err(|e| StoreError::PassphraseGeneration(e))?;

        Ok(GeneratedPassphrases {
            passwords,
            password_handler: self.password_handler,
        })
    }
}

pub struct GeneratedPassphrases<'a> {
    passwords: Vec<String>,
    password_handler: Box<dyn 'a + FnOnce(String) -> Result<(), StoreError>>,
}

impl<'a> GeneratedPassphrases<'a> {
    pub fn passphrases(&self) -> Vec<(usize, String)> {
        self.passwords.clone().into_iter().enumerate().collect()
    }

    pub fn select(mut self, index: usize) -> Result<(), StoreError> {
        if index >= self.passwords.len() {
            return Err(StoreError::PassphraseIndex(index));
        }
        let passphrase = self.passwords.remove(index);
        (self.password_handler)(passphrase)
    }
}
