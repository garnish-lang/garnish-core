#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageBlock {
    pub(crate) cursor: usize,
    pub(crate) size: usize,
    pub(crate) start: usize,
    pub(crate) settings: StorageSettings,
}

impl Default for StorageBlock {
    fn default() -> Self {
        Self { cursor: 0, size: 0, start: 0, settings: StorageSettings::default() }
    }
}

impl StorageBlock {
    pub fn new(size: usize, settings: StorageSettings) -> Self {
        Self { cursor: 0, size, start: 0, settings }
    }

    pub fn next_size(&self) -> usize {
        match self.settings.reallocation_strategy() {
            ReallocationStrategy::FixedSize(size) => self.size + size,
            ReallocationStrategy::Multiplicative(multiplier) => self.size * multiplier,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSettings {
    pub(crate) initial_size: usize,
    pub(crate) max_items: usize,
    pub(crate) reallocation_strategy: ReallocationStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReallocationStrategy {
    FixedSize(usize),
    Multiplicative(usize),
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self { initial_size: 10, max_items: usize::MAX, reallocation_strategy: ReallocationStrategy::FixedSize(10) }
    }
}

impl StorageSettings {
    pub fn new(initial_size: usize, max_items: usize, reallocation_strategy: ReallocationStrategy) -> Self {
        Self { initial_size, max_items, reallocation_strategy }
    }

    pub fn initial_size(&self) -> usize {
        self.initial_size
    }

    pub fn max_items(&self) -> usize {
        self.max_items
    }

    pub fn reallocation_strategy(&self) -> ReallocationStrategy {
        self.reallocation_strategy.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::basic::storage::{ReallocationStrategy, StorageBlock, StorageSettings};

    #[test]
    pub fn next_size_fixed_size() {
        let storage = StorageBlock::new(10, StorageSettings::new(10, 10, ReallocationStrategy::FixedSize(10)));
        let next_size = storage.next_size();
        assert_eq!(next_size, 20);
    }

    #[test]
    pub fn next_size_multiplicative() {
        let storage = StorageBlock::new(10, StorageSettings::new(10, 10, ReallocationStrategy::Multiplicative(2)));
        let next_size = storage.next_size();
        assert_eq!(next_size, 20);
    }
}