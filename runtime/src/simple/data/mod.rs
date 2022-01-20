mod types;
mod utilities;
mod enum_data;

pub use types::*;
pub use utilities::*;
pub use enum_data::*;

use crate::runtime::types::ExpressionDataType;
use std::any::Any;
use std::fmt::Debug;

pub trait SimpleData: Any + Debug + std::hash::Hash {
    fn get_type(&self) -> ExpressionDataType;
}

#[derive(Debug)]
pub(crate) struct AnyData {
    pub(crate) data: Box<dyn Any>,
    data_type: ExpressionDataType,
}

impl AnyData {
    pub(crate) fn new(data: Box<dyn Any>, data_type: ExpressionDataType) -> Self {
        AnyData { data , data_type}
    }

    pub(crate) fn get_data_type(&self) -> ExpressionDataType {
        self.data_type
    }
}

pub(crate) trait AsAnyData {
    fn as_any_data(self) -> AnyData;
}

impl<T: SimpleData> AsAnyData for T {
    fn as_any_data(self) -> AnyData {
        let t = self.get_type();
        AnyData::new(Box::new(self), t)
    }
}

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
