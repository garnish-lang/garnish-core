mod enum_data;

pub use enum_data::*;

#[derive(Debug, Eq, PartialEq)]
pub struct SimpleDataList {
    list: Vec<SimpleDataEnum>,
}

impl Default for SimpleDataList {
    fn default() -> Self {
        SimpleDataList::new().append(SimpleDataEnum::Unit).append(SimpleDataEnum::False).append(SimpleDataEnum::True)
    }
}

impl SimpleDataList {
    pub fn new() -> Self {
        SimpleDataList { list: vec![] }
    }

    pub fn append(mut self, item: SimpleDataEnum) -> Self {
        self.list.push(item);
        self
    }

    pub fn push(&mut self, item: SimpleDataEnum) {
        self.list.push(item);
    }

    pub(crate) fn get(&self, index: usize) -> Option<&SimpleDataEnum> {
        self.list.get(index)
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}
