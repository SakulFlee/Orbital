use std::fmt::Display;

#[derive(Debug, Clone, Default)]
pub struct CacheChange {
    pub before: u64,
    pub after: u64,
}

impl CacheChange {
    pub fn change(&self) -> u64 {
        if self.before > self.after {
            panic!("Cache change before cannot be bigger than after!");
        }

        self.before - self.after
    }
}

impl Display for CacheChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Change\n\tBefore: {} entries\n\tAfter: {} entries\n\t\tChange: -{} entries",
            self.before,
            self.after,
            self.change()
        )
    }
}
