mod types;
mod utilities;

pub use types::*;
pub use utilities::*;

use crate::runtime::types::ExpressionDataType;
use std::any::Any;
use std::fmt::Debug;
use std::slice::Iter;

pub trait SimpleData: Any + Debug + std::hash::Hash {
    fn get_type(&self) -> ExpressionDataType;
}

#[derive(Debug)]
pub struct AnyData {
    pub(crate) data: Box<dyn Any>,
}

impl AnyData {
    pub(crate) fn new(data: Box<dyn Any>) -> Self {
        AnyData { data }
    }
}

pub(crate) trait AsAnyData {
    fn as_any_data(self) -> AnyData;
}

impl<T: SimpleData> AsAnyData for T {
    fn as_any_data(self) -> AnyData {
        AnyData::new(Box::new(self))
    }
}

#[derive(Debug)]
pub struct SimpleDataList {
    list: Vec<AnyData>,
}

impl Default for SimpleDataList {
    fn default() -> Self {
        SimpleDataList::new().append(UnitData::new()).append(FalseData::new()).append(TrueData::new())
    }
}

impl SimpleDataList {
    pub fn new() -> Self {
        SimpleDataList { list: vec![] }
    }

    pub fn append<T: SimpleData>(mut self, item: T) -> Self {
        self.list.push(item.as_any_data());
        self
    }

    pub fn push<T: SimpleData>(&mut self, item: T) {
        self.list.push(item.as_any_data());
    }

    pub fn get(&self, index: usize) -> Option<&AnyData> {
        self.list.get(index)
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn iter(&self) -> Iter<'_, AnyData> {
        self.list.iter()
    }
}

impl PartialEq<SimpleDataList> for SimpleDataList {
    fn eq(&self, other: &SimpleDataList) -> bool {
        if self.list.len() != other.list.len() {
            return false;
        }

        let mut equal = true;
        for i in 0..self.list.len() {
            match (self.list.get(i), other.list.get(i)) {
                (Some(left), Some(right)) => {
                    if !data_equal(&left.data, &right.data) {
                        equal = false;
                        break;
                    }
                }
                _ => {
                    equal = false;
                    break;
                }
            }
        }

        equal
    }
}

