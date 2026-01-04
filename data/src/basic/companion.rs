use garnish_lang_traits::{GarnishDataType, Instruction};
use std::fmt::Debug;

use crate::{BasicDataCustom, BasicGarnishData, DataError};

pub trait BasicDataCompanion<T>: Clone + Debug + PartialEq + Eq + PartialOrd where T: BasicDataCustom {
    fn resolve(data: &mut BasicGarnishData<T, Self>, symbol: u64) -> Result<bool, DataError>;
    fn apply(data: &mut BasicGarnishData<T, Self>, external_value: usize, input_addr: usize) -> Result<bool, DataError>;
    fn defer_op(data: &mut BasicGarnishData<T, Self>, operation: Instruction, left: (GarnishDataType, usize), right: (GarnishDataType, usize)) -> Result<bool, DataError>;
}