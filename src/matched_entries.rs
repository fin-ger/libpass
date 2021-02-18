use crate::{Entry, RecursiveTraversal};

pub struct MatchedEntries<'a, 'b> {
    pattern: &'b str,
    traverser: RecursiveTraversal<'a>,
}

impl<'a, 'b> MatchedEntries<'a, 'b> {
    pub(crate) fn new(pattern: &'b str, traverser: RecursiveTraversal<'a>) -> Self {
        Self { pattern, traverser }
    }
}

impl<'a, 'b> Iterator for MatchedEntries<'a, 'b> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.traverser.next() {
            if next
                .path()
                .to_string_lossy()
                .to_lowercase()
                .find(&self.pattern.to_lowercase())
                .is_some()
            {
                return Some(next);
            }
        }

        None
    }
}
