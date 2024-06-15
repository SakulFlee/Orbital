use std::fmt::Display;

#[derive(Debug, Clone, Default)]
pub struct CacheChange {
    pub before: usize,
    pub after: usize,
}

impl CacheChange {
    /// Calculate the change between `before` and `after`
    ///
    /// # Safety
    /// This function can overflow, but not panic.
    /// If `before` and/or `after` are bigger than [isize],
    /// this will return wrong results as the number will wrap around.
    pub fn change(&self) -> isize {
        self.before as isize - self.after as isize
    }
}

impl Display for CacheChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let change = self.change();
        let sign = if change == 0 {
            ""
        } else if change > 0 {
            "-"
        } else {
            "+"
        };

        write!(
            f,
            "Cache Change\n\tBefore: {} entries\n\tAfter: {} entries\n\t\tChange: {}{} entries",
            self.before, self.after, sign, change
        )
    }
}
