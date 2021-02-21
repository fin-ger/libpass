use passwords::{analyzer, scorer, AnalyzedPassword};

pub struct AnalyzedPassphrase {
    analyzed_password: AnalyzedPassword,
}

impl AnalyzedPassphrase {
    pub fn new(passphrase: &str) -> Self {
        Self {
            analyzed_password: analyzer::analyze(passphrase),
        }
    }

    pub fn length(&self) -> usize {
        self.analyzed_password.length()
    }

    pub fn space_count(&self) -> usize {
        self.analyzed_password.spaces_count()
    }

    pub fn numbers_count(&self) -> usize {
        self.analyzed_password.numbers_count()
    }

    pub fn lowercase_letters_count(&self) -> usize {
        self.analyzed_password.lowercase_letters_count()
    }

    pub fn uppercase_letters_count(&self) -> usize {
        self.analyzed_password.uppercase_letters_count()
    }

    pub fn symbols_count(&self) -> usize {
        self.analyzed_password.symbols_count()
    }

    pub fn other_characters_count(&self) -> usize {
        self.analyzed_password.other_characters_count()
    }

    pub fn consecutive_count(&self) -> usize {
        self.analyzed_password.consecutive_count()
    }

    pub fn non_consecutive_count(&self) -> usize {
        self.analyzed_password.non_consecutive_count()
    }

    pub fn score(&self) -> f64 {
        scorer::score(&self.analyzed_password)
    }
}
