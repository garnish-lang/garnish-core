
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
        Self { initial_size: 1000, max_items: usize::MAX, reallocation_strategy: ReallocationStrategy::Multiplicative(2) }
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