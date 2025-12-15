
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSettings {
    max_items: usize,
    reallocation_strategy: ReallocationStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReallocationStrategy {
    FixedSize(usize),
    Multiplicative(usize),
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self { max_items: usize::MAX, reallocation_strategy: ReallocationStrategy::Multiplicative(2) }
    }
}