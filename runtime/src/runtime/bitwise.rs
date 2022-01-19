use crate::{GarnishLangRuntimeData, GarnishNumber, RuntimeError};
use crate::runtime::arithmetic::{perform_op, perform_unary_op};

pub fn bitwise_not<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_unary_op(this, "bitwise not", Data::Number::bitwise_not)
}

pub fn bitwise_and<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "bitwise and", Data::Number::bitwise_and)
}

pub fn bitwise_or<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "bitwise or", Data::Number::bitwise_or)
}

pub fn bitwise_xor<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "bitwise xor", Data::Number::bitwise_xor)
}

pub fn bitwise_shift_left<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "bitwise shift left", Data::Number::bitwise_shift_left)
}

pub fn bitwise_shift_right<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    perform_op(this, "bitwise shift right", Data::Number::bitwise_shift_right)
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData};

    #[test]
    fn bitwise_not() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();

        runtime.bitwise_not().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), !10);
    }

    #[test]
    fn bitwise_and() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_and().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 & 20);
    }

    #[test]
    fn bitwise_or() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_or().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 | 20);
    }

    #[test]
    fn bitwise_xor() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(20).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_xor().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 ^ 20);
    }

    #[test]
    fn bitwise_shift_left() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(3).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_shift_left().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 << 3);
    }

    #[test]
    fn bitwise_shift_right() {
        let mut runtime = SimpleRuntimeData::new();

        let int1 = runtime.add_number(10).unwrap();
        let int2 = runtime.add_number(3).unwrap();
        let new_data_start = runtime.get_data_len();

        runtime.push_register(int1).unwrap();
        runtime.push_register(int2).unwrap();

        runtime.bitwise_shift_right().unwrap();

        assert_eq!(runtime.get_register(0).unwrap(), new_data_start);
        assert_eq!(runtime.get_number(new_data_start).unwrap(), 10 >> 3);
    }
}
