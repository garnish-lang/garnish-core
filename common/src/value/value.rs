use crate::byte_vec::DataVecWriter;
use crate::data_type::DataType;
use crate::error::Result;
use crate::utils::insert_associative_list_keys;
use crate::value::range::{
    RANGE_END_EXCLUSIVE, RANGE_HAS_STEP, RANGE_OPEN_END, RANGE_OPEN_START, RANGE_START_EXCLUSIVE,
};
use crate::value::ExpressionValueRef;
use crate::{Error, SIZE_LENGTH};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use unicode_segmentation::UnicodeSegmentation;

pub struct ExpressionValue {
    data: Vec<u8>,
    symbol_table: HashMap<String, usize>,
    error: Option<Error>,
    start: usize,
}

impl<T> From<T> for ExpressionValue
where
    T: ExpressionValueBuilder,
{
    fn from(builder: T) -> ExpressionValue {
        let mut data: Vec<u8> = vec![];
        let mut symbol_table = HashMap::new();
        let mut start = 0;
        symbol_table.insert("".to_string(), 0);

        let error = match builder.write_data(&mut data, &mut symbol_table) {
            Ok(s) => {
                start = s;
                None
            }
            Err(e) => Some(e),
        };

        return ExpressionValue {
            data,
            symbol_table,
            error,
            start,
        };
    }
}

impl ExpressionValue {
    pub fn reference(&self) -> Result<ExpressionValueRef> {
        ExpressionValueRef::new_with_start(&self.data[..], Some(&self.symbol_table), self.start)
    }
}

pub trait ExpressionValueBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize>;
}

pub struct UnitBuilder {}

impl ExpressionValueBuilder for UnitBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        _symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        let pos = v.len();
        v.push(DataType::Unit.try_into().unwrap());
        Ok(pos)
    }
}

pub struct IntegerBuilder {
    num: i32,
}

impl ExpressionValueBuilder for IntegerBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        _symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        let pos = v.len();

        DataVecWriter::new(v)
            .push_data_type(DataType::Integer)
            .push_integer(self.num);

        Ok(pos)
    }
}

pub struct FloatBuilder {
    num: f32,
}

impl ExpressionValueBuilder for FloatBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        _symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        let pos = v.len();

        DataVecWriter::new(v)
            .push_data_type(DataType::Float)
            .push_float(self.num);

        Ok(pos)
    }
}

pub struct CharacterBuilder {
    s: String,
}

impl ExpressionValueBuilder for CharacterBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        _symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        let pos = v.len();

        DataVecWriter::new(v)
            .push_data_type(DataType::Character)
            .push_byte_size(self.s.len() as u8)
            .push_character(&self.s);

        Ok(pos)
    }
}

pub struct CharacterListBuilder {
    s: String,
}

impl ExpressionValueBuilder for CharacterListBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        let mut refs = vec![];

        for c in self.s.graphemes(true) {
            let index = ExpressionValue::character(c.into()).write_data(v, symbol_table)?;
            refs.push(index);
        }

        let pos = v.len();

        let mut writer = DataVecWriter::new(v)
            .push_data_type(DataType::CharacterList)
            .push_size(refs.len());

        for r in refs {
            writer = writer.push_size(r);
        }

        Ok(pos)
    }
}

fn write_symbol_like(
    s: &String,
    data_type: DataType,
    v: &mut Vec<u8>,
    symbol_table: &mut HashMap<String, usize>,
) -> Result<usize> {
    let pos = v.len();
    let symbol_value = if s.is_empty() {
        0
    } else {
        let value = match symbol_table.get(s) {
            None => symbol_table.len(),
            Some(v) => *v,
        };

        // store location of symbol value for updating later
        symbol_table.insert(s.clone(), value);

        value
    };

    DataVecWriter::new(v)
        .push_data_type(data_type)
        .push_size(symbol_value);

    Ok(pos)
}

pub struct ExpressionBuilder {
    s: String,
}

impl ExpressionValueBuilder for ExpressionBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        write_symbol_like(&self.s, DataType::Expression, v, symbol_table)
    }
}

pub struct ExternalMethodBuilder {
    s: String,
}

impl ExpressionValueBuilder for ExternalMethodBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        write_symbol_like(&self.s, DataType::ExternalMethod, v, symbol_table)
    }
}

pub struct SymbolBuilder {
    s: String,
}

impl ExpressionValueBuilder for SymbolBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        write_symbol_like(&self.s, DataType::Symbol, v, symbol_table)
    }
}

pub struct RangeBuilder<T, S = T>
where
    T: ExpressionValueBuilder,
    S: ExpressionValueBuilder,
{
    start: Option<T>,
    end: Option<T>,
    step: Option<S>,
    flags: u8,
}

impl<T, S> RangeBuilder<T, S>
where
    T: ExpressionValueBuilder,
    S: ExpressionValueBuilder,
{
    pub fn exclude_end(mut self) -> Self {
        self.flags |= RANGE_END_EXCLUSIVE;
        self
    }

    pub fn exclude_start(mut self) -> Self {
        self.flags |= RANGE_START_EXCLUSIVE;
        self
    }

    pub fn with_step(mut self, step: S) -> Self {
        self.flags |= RANGE_HAS_STEP;
        self.step = Some(step);
        self
    }
}

impl<T, S> ExpressionValueBuilder for RangeBuilder<T, S>
where
    T: ExpressionValueBuilder,
    S: ExpressionValueBuilder,
{
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        let mut flags = self.flags;

        let min = match &self.start {
            Some(min) => Some(min.write_data(v, symbol_table)?),
            None => {
                flags |= RANGE_OPEN_START;
                None
            }
        };

        let max = match &self.end {
            Some(max) => Some(max.write_data(v, symbol_table)?),
            None => {
                flags |= RANGE_OPEN_END;
                None
            }
        };

        let step = match &self.step {
            Some(step) => Some(step.write_data(v, symbol_table)?),
            None => None,
        };

        let pos = v.len();
        let writer = DataVecWriter::new(v)
            .push_data_type(DataType::Range)
            .push_byte_size(flags);

        let writer = match min {
            Some(m) => writer.push_size(m),
            None => writer,
        };

        let writer = match max {
            Some(m) => writer.push_size(m),
            None => writer,
        };

        match step {
            Some(s) => writer.push_size(s),
            None => writer,
        };

        Ok(pos)
    }
}

pub struct PairBuilder<T, U>
where
    T: ExpressionValueBuilder,
    U: ExpressionValueBuilder,
{
    key: T,
    value: U,
}

impl<T, U> ExpressionValueBuilder for PairBuilder<T, U>
where
    T: ExpressionValueBuilder,
    U: ExpressionValueBuilder,
{
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        let key_ref = self.key.write_data(v, symbol_table)?;
        let value_ref = self.value.write_data(v, symbol_table)?;
        let pos = v.len();

        DataVecWriter::new(v)
            .push_data_type(DataType::Pair)
            .push_size(key_ref)
            .push_size(value_ref);

        Ok(pos)
    }
}

pub struct PartialBuilder<T, U>
where
    T: ExpressionValueBuilder,
    U: ExpressionValueBuilder,
{
    base: T,
    value: U,
}

impl<T, U> ExpressionValueBuilder for PartialBuilder<T, U>
where
    T: ExpressionValueBuilder,
    U: ExpressionValueBuilder,
{
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        let key_ref = self.base.write_data(v, symbol_table)?;
        let value_ref = self.value.write_data(v, symbol_table)?;
        let pos = v.len();

        DataVecWriter::new(v)
            .push_data_type(DataType::Partial)
            .push_size(key_ref)
            .push_size(value_ref);

        Ok(pos)
    }
}

pub struct AssociativeListBuilder {
    elements: Vec<Box<dyn ExpressionValueBuilder>>,
}

impl AssociativeListBuilder {
    pub fn add<T>(mut self, element: T) -> Self
    where
        T: ExpressionValueBuilder,
        T: 'static,
    {
        self.elements.push(Box::new(element));
        self
    }
}

impl ExpressionValueBuilder for AssociativeListBuilder {
    fn write_data(
        &self,
        v: &mut Vec<u8>,
        symbol_table: &mut HashMap<String, usize>,
    ) -> Result<usize> {
        // push elements
        let mut refs: Vec<usize> = vec![];
        let mut key_refs: Vec<usize> = vec![];
        for element in self.elements.iter() {
            let value_ref = element.write_data(v, symbol_table)?;
            if DataType::try_from(v[value_ref])? == DataType::Pair {
                let key_ref = v[value_ref + 1] as usize;
                let key_type = DataType::try_from(v[key_ref])?;
                if key_type == DataType::Symbol || key_type == DataType::CharacterList {
                    key_refs.push(value_ref);
                }
            }
            refs.push(value_ref);
        }

        let pos = v.len();

        let mut writer = DataVecWriter::new(v)
            .push_data_type(DataType::List)
            .push_size(refs.len())
            .push_size(key_refs.len());

        // maybe reverse to be consistent how lists are created at runtime?
        for r in refs.iter() {
            writer = writer.push_size(*r);
        }

        let range_start = v.len();
        // initialize key area
        // u32 is size of 'size' in writer
        for _i in 0..(key_refs.len() * SIZE_LENGTH) {
            v.push(0);
        }

        let key_area_range = range_start..(range_start + key_refs.len());

        insert_associative_list_keys(v, key_area_range, &key_refs).unwrap();

        Ok(pos)
    }
}

pub struct SliceBuilder {
    list: AssociativeListBuilder,
    range: RangeBuilder<IntegerBuilder>,
}

impl ExpressionValueBuilder for SliceBuilder {
    fn write_data(&self, v: &mut Vec<u8>, symbol_table: &mut HashMap<String, usize>) -> Result<usize> {
        let list_ref = self.list.write_data(v, symbol_table)?;
        let range_ref = self.range.write_data(v, symbol_table)?;

        let pos = v.len();

        DataVecWriter::new(v)
            .push_data_type(DataType::Slice)
            .push_size(list_ref)
            .push_size(range_ref);

        Ok(pos)
    }
}

pub struct LinkBuilder<T, U>
where
    T: ExpressionValueBuilder,
    U: ExpressionValueBuilder
{
    first: T,
    second: U,
}

impl<T, U> ExpressionValueBuilder for LinkBuilder<T, U>
where
    T: ExpressionValueBuilder,
    U: ExpressionValueBuilder,
{
    fn write_data(&self, v: &mut Vec<u8>, symbol_table: &mut HashMap<String, usize>) -> Result<usize> {
        let first_ref = self.first.write_data(v, symbol_table)?;
        let second_ref = self.second.write_data(v, symbol_table)?;

        let pos = v.len();

        DataVecWriter::new(v)
            .push_data_type(DataType::Link)
            .push_size(first_ref)
            .push_size(first_ref)
            .push_size(second_ref);

        Ok(pos)
    }
}

impl ExpressionValue {
    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_symbol_table(&self) -> &HashMap<String, usize> {
        &self.symbol_table
    }

    pub fn get_error(&self) -> Option<Error> {
        self.error.clone()
    }

    pub fn unit() -> UnitBuilder {
        UnitBuilder {}
    }

    pub fn integer(num: i32) -> IntegerBuilder {
        IntegerBuilder { num }
    }

    pub fn float(num: f32) -> FloatBuilder {
        FloatBuilder { num }
    }

    pub fn character(s: String) -> CharacterBuilder {
        CharacterBuilder { s }
    }

    pub fn character_list(s: String) -> CharacterListBuilder {
        CharacterListBuilder { s }
    }

    pub fn expression<T>(s: T) -> ExpressionBuilder
    where
        T: ToString,
    {
        ExpressionBuilder { s: s.to_string() }
    }

    pub fn external_method<T>(s: T) -> ExternalMethodBuilder
    where
        T: ToString,
    {
        ExternalMethodBuilder { s: s.to_string() }
    }

    pub fn symbol<T>(s: T) -> SymbolBuilder
    where
        T: ToString,
    {
        SymbolBuilder { s: s.to_string() }
    }

    pub fn integer_range(min: Option<i32>, max: Option<i32>) -> RangeBuilder<IntegerBuilder> {
        let builder = RangeBuilder {
            start: match min {
                Some(min) => Some(ExpressionValue::integer(min)),
                None => None,
            },
            end: match max {
                Some(max) => Some(ExpressionValue::integer(max)),
                None => None,
            },
            step: None,
            flags: 0,
        };
        builder
    }

    pub fn float_range(min: Option<f32>, max: Option<f32>) -> RangeBuilder<FloatBuilder> {
        let builder = RangeBuilder {
            start: match min {
                Some(min) => Some(ExpressionValue::float(min)),
                None => None,
            },
            end: match max {
                Some(max) => Some(ExpressionValue::float(max)),
                None => None,
            },
            step: None,
            flags: 0,
        };
        builder
    }

    pub fn char_range(min: Option<char>, max: Option<char>) -> RangeBuilder<CharacterBuilder> {
        RangeBuilder {
            start: match min {
                Some(min) => Some(ExpressionValue::character(min.to_string())),
                None => None,
            },
            end: match max {
                Some(max) => Some(ExpressionValue::character(max.to_string())),
                None => None,
            },
            step: None,
            flags: 0,
        }
    }

    pub fn pair<T, U>(key: T, value: U) -> PairBuilder<T, U>
    where
        T: ExpressionValueBuilder,
        U: ExpressionValueBuilder,
    {
        PairBuilder { key, value }
    }

    pub fn partial_expression<V>(expression: &str, value: V) -> PartialBuilder<ExpressionBuilder, V>
    where
        V: ExpressionValueBuilder,
    {
        PartialBuilder {
            base: ExpressionValue::expression(expression),
            value,
        }
    }

    pub fn partial_external_method<V>(
        expression: &str,
        value: V,
    ) -> PartialBuilder<ExternalMethodBuilder, V>
    where
        V: ExpressionValueBuilder,
    {
        PartialBuilder {
            base: ExpressionValue::external_method(expression),
            value,
        }
    }

    pub fn list_slice(list: AssociativeListBuilder, range: RangeBuilder<IntegerBuilder>) -> SliceBuilder {
        SliceBuilder { list, range }
    }

    pub fn link<T, U>(first: T, second: U) -> LinkBuilder<T, U>
    where
        T: ExpressionValueBuilder,
        U: ExpressionValueBuilder,
    {
        LinkBuilder { first, second }
    }

    pub fn list() -> AssociativeListBuilder {
        AssociativeListBuilder { elements: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_vec::DataVecWriter;
    use crate::data_type::DataType;
    use crate::value::range::{
        RANGE_END_EXCLUSIVE, RANGE_HAS_STEP, RANGE_OPEN_END, RANGE_OPEN_START,
        RANGE_START_EXCLUSIVE,
    };
    use crate::value::ExpressionValueBuilder;
    use crate::ExpressionValue;
    use std::collections::HashMap;
    use std::convert::TryInto;

    #[test]
    fn unit_vec() {
        let result: ExpressionValue = ExpressionValue::unit().into();
        let expected: Vec<u8> = vec![DataType::Unit.try_into().unwrap()];
        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn integer_vec() {
        let result: ExpressionValue = ExpressionValue::integer(-10).into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(-10);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn float_vec() {
        let result: ExpressionValue = ExpressionValue::float(-3.14).into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Float)
            .push_float(-3.14);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn character_vec_ascii() {
        let result: ExpressionValue = ExpressionValue::character(String::from("z")).into();
        let mut expected: Vec<u8> = vec![];

        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Character)
            .push_byte_size(1)
            .push_character("z");

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn character_vec_unicode() {
        let result: ExpressionValue = ExpressionValue::character(String::from("🐼")).into();
        let mut expected: Vec<u8> = vec![];

        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Character)
            .push_byte_size(4)
            .push_character("🐼");

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn character_vec_grapheme_cluster() {
        let result: ExpressionValue = ExpressionValue::character(String::from("ö̲")).into();
        let mut expected: Vec<u8> = vec![];

        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Character)
            .push_byte_size(5)
            .push_character("ö̲");

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn character_list_value() {
        let result: ExpressionValue = ExpressionValue::character_list(String::from("z🐼ö̲")).into();
        let mut expected: Vec<u8> = vec![];

        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Character) // 0
            .push_byte_size(1) // 1
            .push_character("z") // 2
            .push_data_type(DataType::Character) // 3
            .push_byte_size(4) // 4
            .push_character("🐼") // 5
            .push_data_type(DataType::Character) // 9
            .push_byte_size(5) // 10
            .push_character("ö̲") // 11
            .push_data_type(DataType::CharacterList) // 16
            .push_size(3)
            .push_size(0)
            .push_size(3)
            .push_size(9);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn expression() {
        let result: ExpressionValue = ExpressionValue::expression("Bears").into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Expression)
            .push_size(1);

        assert_eq!(result.get_data().to_owned(), expected);
        assert_eq!(result.symbol_table.get("Bears").unwrap().to_owned(), 1);
    }

    #[test]
    fn external_method() {
        let result: ExpressionValue = ExpressionValue::external_method("Cats").into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::ExternalMethod)
            .push_size(1);

        assert_eq!(result.get_data().to_owned(), expected);
        assert_eq!(result.symbol_table.get("Cats").unwrap().to_owned(), 1);
    }

    #[test]
    fn symbol_vec() {
        let result: ExpressionValue = ExpressionValue::symbol("my_symbol").into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Symbol)
            .push_size(1);

        assert_eq!(result.get_data().to_owned(), expected);
        assert_eq!(result.symbol_table.get("my_symbol").unwrap().to_owned(), 1);
    }

    #[test]
    fn empty_symbol() {
        let result: ExpressionValue = ExpressionValue::symbol("").into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Symbol)
            .push_size(0);

        assert_eq!(result.get_data().to_owned(), expected);
        assert_eq!(*result.symbol_table.get("").unwrap(), 0);
    }

    #[test]
    fn exclusive_start_integer_range() {
        let result: ExpressionValue = ExpressionValue::integer_range(Some(10), Some(20))
            .exclude_start()
            .into();

        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(10)
            .push_data_type(DataType::Integer)
            .push_integer(20)
            .push_data_type(DataType::Range)
            .push_byte_size(RANGE_START_EXCLUSIVE)
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn exclusive_end_integer_range() {
        let result: ExpressionValue = ExpressionValue::integer_range(Some(10), Some(20))
            .exclude_end()
            .into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(10)
            .push_data_type(DataType::Integer)
            .push_integer(20)
            .push_data_type(DataType::Range)
            .push_byte_size(RANGE_END_EXCLUSIVE)
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn exclusive_start_and_end_integer_range() {
        let result: ExpressionValue = ExpressionValue::integer_range(Some(10), Some(20))
            .exclude_start()
            .exclude_end()
            .into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(10)
            .push_data_type(DataType::Integer)
            .push_integer(20)
            .push_data_type(DataType::Range)
            .push_byte_size(RANGE_END_EXCLUSIVE | RANGE_START_EXCLUSIVE)
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn open_start_integer_range() {
        let result: ExpressionValue = ExpressionValue::integer_range(None, Some(20)).into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(20)
            .push_data_type(DataType::Range)
            .push_byte_size(RANGE_OPEN_START)
            .push_size(0);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn open_end_integer_range() {
        let result: ExpressionValue = ExpressionValue::integer_range(Some(10), None).into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(10)
            .push_data_type(DataType::Range)
            .push_byte_size(RANGE_OPEN_END)
            .push_size(0);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn open_start_and_end_integer_range() {
        let result: ExpressionValue = ExpressionValue::integer_range(None, None).into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Range)
            .push_byte_size(RANGE_OPEN_START | RANGE_OPEN_END);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn integer_range_with_step() {
        let result: ExpressionValue = ExpressionValue::integer_range(Some(10), Some(20))
            .with_step(ExpressionValue::integer(2))
            .into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(10)
            .push_data_type(DataType::Integer)
            .push_integer(20)
            .push_data_type(DataType::Integer)
            .push_integer(2)
            .push_data_type(DataType::Range)
            .push_byte_size(RANGE_HAS_STEP)
            .push_size(0)
            .push_size(5)
            .push_size(10);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn char_range() {
        let result: ExpressionValue = ExpressionValue::char_range(Some('a'), Some('z')).into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Character)
            .push_byte_size(1)
            .push_character("a")
            .push_data_type(DataType::Character)
            .push_byte_size(1)
            .push_character("z")
            .push_data_type(DataType::Range)
            .push_byte_size(0)
            .push_size(0)
            .push_size(3);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn float_range() {
        let result: ExpressionValue = ExpressionValue::float_range(Some(1.5), Some(4.6)).into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Float)
            .push_float(1.5)
            .push_data_type(DataType::Float)
            .push_float(4.6)
            .push_data_type(DataType::Range)
            .push_byte_size(0)
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn pair_same_type_vec() {
        let result: ExpressionValue =
            ExpressionValue::pair(ExpressionValue::integer(50), ExpressionValue::integer(100))
                .into();
        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer)
            .push_integer(50)
            .push_data_type(DataType::Integer)
            .push_integer(100)
            .push_data_type(DataType::Pair)
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn pair_different_type() {
        let result: ExpressionValue = ExpressionValue::pair(
            ExpressionValue::symbol("symbol"),
            ExpressionValue::integer(50),
        )
        .into();

        let mut expected = vec![];

        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Symbol)
            .push_size(1)
            .push_data_type(DataType::Integer)
            .push_integer(50)
            .push_data_type(DataType::Pair)
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn partial_expression() {
        let result: ExpressionValue =
            ExpressionValue::partial_expression("my_expression", ExpressionValue::integer(100))
                .into();

        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Expression)
            .push_integer(1)
            .push_data_type(DataType::Integer)
            .push_integer(100)
            .push_data_type(DataType::Partial)
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn partial_external_method() {
        let result: ExpressionValue = ExpressionValue::partial_external_method(
            "my_external_method",
            ExpressionValue::integer(100),
        )
        .into();

        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::ExternalMethod)
            .push_integer(1)
            .push_data_type(DataType::Integer)
            .push_integer(100)
            .push_data_type(DataType::Partial)
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn list_slice() {
        let result: ExpressionValue = ExpressionValue::list_slice(
            ExpressionValue::list()
                .add(ExpressionValue::integer(10))
                .add(ExpressionValue::integer(20))
                .add(ExpressionValue::integer(30)),
            ExpressionValue::integer_range(Some(0), Some(1))
        ).into();

        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer) // 0
            .push_integer(10)
            .push_data_type(DataType::Integer) // 5
            .push_integer(20)
            .push_data_type(DataType::Integer) // 10
            .push_integer(30)
            .push_data_type(DataType::List) // 15
            .push_size(3) // 16
            .push_size(0)
            .push_size(0)
            .push_size(5)
            .push_size(10)
            .push_data_type(DataType::Integer) // 36
            .push_integer(0)
            .push_data_type(DataType::Integer) // 41
            .push_integer(1)
            .push_data_type(DataType::Range) // 46
            .push_byte_size(0)
            .push_size(36)
            .push_size(41)
            .push_data_type(DataType::Slice)
            .push_size(15)
            .push_size(46);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn link() {
        let result: ExpressionValue = ExpressionValue::link(
            ExpressionValue::integer(10),
            ExpressionValue::integer(20)
        ).into();

        let mut expected: Vec<u8> = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Integer) // 0
            .push_integer(10)
            .push_data_type(DataType::Integer) // 5
            .push_integer(20)
            .push_data_type(DataType::Link) // 10
            .push_size(0) 
            .push_size(0)
            .push_size(5);

        assert_eq!(result.get_data().to_owned(), expected);
    }

    #[test]
    fn associative_list_pairs_only() {
        let result: ExpressionValue = ExpressionValue::list()
            .add(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer_range(Some(10), Some(20)).exclude_end(),
            ))
            .add(ExpressionValue::pair(
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(20),
            ))
            .into();

        let mut expected = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Symbol) // 0
            .push_size(1) // 1
            .push_data_type(DataType::Integer) // 5
            .push_integer(10)
            .push_data_type(DataType::Integer) // 10
            .push_integer(20)
            .push_data_type(DataType::Range) // 15
            .push_byte_size(RANGE_END_EXCLUSIVE)
            .push_integer(5) // 7
            .push_integer(10) // 11
            .push_data_type(DataType::Pair) // 25
            .push_size(0) // 15
            .push_size(15) // 19
            .push_data_type(DataType::Symbol) // 34
            .push_size(2) // 24
            .push_data_type(DataType::Integer) // 39
            .push_integer(20) // 29
            .push_data_type(DataType::Pair) // 44
            .push_size(34) // 34
            .push_size(39) // 39
            .push_data_type(DataType::List) // 53
            .push_size(2) // 44
            .push_size(2)
            .push_size(25)
            .push_size(44)
            .push_size(44)
            .push_size(25);

        assert_eq!(result.data, expected);
    }

    #[test]
    fn associative_list_of_associative_lists() {
        let result: ExpressionValue = ExpressionValue::list()
            .add(
                ExpressionValue::list()
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("bear"),
                        ExpressionValue::integer(10),
                    ))
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("cat"),
                        ExpressionValue::integer(20),
                    )),
            )
            .add(
                ExpressionValue::list()
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("dog"),
                        ExpressionValue::integer(30),
                    ))
                    .add(ExpressionValue::pair(
                        ExpressionValue::symbol("rabbit"),
                        ExpressionValue::integer(40),
                    )),
            )
            .into();

        let mut expected = vec![];
        DataVecWriter::new(&mut expected)
            // list 1
            .push_data_type(DataType::Symbol) // 0
            .push_size(1) // 1
            .push_data_type(DataType::Integer) // 5
            .push_integer(10) // 6
            .push_data_type(DataType::Pair) // 10
            .push_size(0) // 11
            .push_size(5) // 15
            .push_data_type(DataType::Symbol) // 19
            .push_size(2) // 20
            .push_data_type(DataType::Integer) // 24
            .push_integer(20) // 25
            .push_data_type(DataType::Pair) // 29
            .push_size(19) // 30
            .push_size(24) // 34
            .push_data_type(DataType::List) // 38
            .push_size(2) // 39
            .push_size(2) // 43
            .push_size(10) // 47
            .push_size(29) // 51
            .push_size(29) // 55
            .push_size(10) // 59
            // list 2
            .push_data_type(DataType::Symbol) // 63
            .push_size(3) // 64
            .push_data_type(DataType::Integer) // 68
            .push_integer(30) // 69
            .push_data_type(DataType::Pair) // 73
            .push_size(63) // 74
            .push_size(68) // 78
            .push_data_type(DataType::Symbol) // 82
            .push_size(4) // 83
            .push_data_type(DataType::Integer) // 87
            .push_integer(40) // 88
            .push_data_type(DataType::Pair) // 92
            .push_size(82) // 93
            .push_size(87) // 97
            .push_data_type(DataType::List) // 101
            .push_size(2) // 102
            .push_size(2) // 106
            .push_size(73) // 110
            .push_size(92) // 114
            .push_size(92) // 118
            .push_size(73) // 122
            // list 3
            .push_data_type(DataType::List)
            .push_size(2)
            .push_size(0)
            .push_size(38)
            .push_size(101);

        assert_eq!(result.data, expected);
    }

    #[test]
    fn associative_list_pairs_string_keys() {
        let result: ExpressionValue = ExpressionValue::list()
            .add(ExpressionValue::pair(
                ExpressionValue::character_list("bear".into()),
                ExpressionValue::integer_range(Some(10), Some(20)).exclude_end(),
            ))
            .add(ExpressionValue::pair(
                ExpressionValue::character_list("cat".into()),
                ExpressionValue::integer(20),
            ))
            .into();

        let mut expected = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Character) // 0
            .push_byte_size(1)
            .push_character("b")
            .push_data_type(DataType::Character) // 3
            .push_byte_size(1)
            .push_character("e")
            .push_data_type(DataType::Character) // 6
            .push_byte_size(1)
            .push_character("a")
            .push_data_type(DataType::Character) // 9
            .push_byte_size(1)
            .push_character("r")
            .push_data_type(DataType::CharacterList) // 12
            .push_size(4) // 13
            .push_size(0) // 17
            .push_size(3) // 21
            .push_size(6) // 25
            .push_size(9) // 29
            .push_data_type(DataType::Integer) // 33
            .push_integer(10)
            .push_data_type(DataType::Integer) // 38
            .push_integer(20)
            .push_data_type(DataType::Range) // 43
            .push_byte_size(RANGE_END_EXCLUSIVE)
            .push_integer(33) // 22
            .push_integer(38) // 26
            .push_data_type(DataType::Pair) // 53
            .push_size(12) // 31
            .push_size(43) // 35
            .push_data_type(DataType::Character) // 62
            .push_byte_size(1)
            .push_character("c")
            .push_data_type(DataType::Character) // 65
            .push_byte_size(1)
            .push_character("a")
            .push_data_type(DataType::Character) // 68
            .push_byte_size(1)
            .push_character("t")
            .push_data_type(DataType::CharacterList) // 71
            .push_size(3) // 72
            .push_size(62) // 76
            .push_size(65) // 80
            .push_size(68) // 84
            .push_data_type(DataType::Integer) // 88
            .push_integer(20) // 61
            .push_data_type(DataType::Pair) // 93
            .push_size(71) // 66
            .push_size(88) // 70
            .push_data_type(DataType::List) // 102
            .push_size(2)
            .push_size(2)
            .push_size(53)
            .push_size(93)
            .push_size(93)
            .push_size(53);

        assert_eq!(result.data, expected);
    }

    #[test]
    fn associative_list_pairs_and_values() {
        let result: ExpressionValue = ExpressionValue::list()
            .add(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer_range(Some(10), Some(20)).exclude_end(),
            ))
            .add(ExpressionValue::pair(
                ExpressionValue::symbol("cats"),
                ExpressionValue::integer(20),
            ))
            .add(ExpressionValue::integer(10))
            .add(ExpressionValue::character_list("rabbit".into()))
            .into();

        let mut expected = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Symbol) // 0
            .push_size(1) // 1
            .push_data_type(DataType::Integer) // 5
            .push_integer(10)
            .push_data_type(DataType::Integer) // 10
            .push_integer(20)
            .push_data_type(DataType::Range) // 15
            .push_byte_size(RANGE_END_EXCLUSIVE)
            .push_integer(5) // 6
            .push_integer(10) // 10
            .push_data_type(DataType::Pair) // 25
            .push_size(0) // 15
            .push_size(15) // 19
            .push_data_type(DataType::Symbol) // 34
            .push_size(2) // 24
            .push_data_type(DataType::Integer) // 39
            .push_integer(20) // 29
            .push_data_type(DataType::Pair) // 44
            .push_size(34) // 34
            .push_size(39) // 38
            .push_data_type(DataType::Integer) // 53
            .push_integer(10) // 43
            .push_data_type(DataType::Character) // 58
            .push_byte_size(1)
            .push_character("r")
            .push_data_type(DataType::Character) // 61
            .push_byte_size(1)
            .push_character("a")
            .push_data_type(DataType::Character) // 64
            .push_byte_size(1)
            .push_character("b")
            .push_data_type(DataType::Character) // 67
            .push_byte_size(1)
            .push_character("b")
            .push_data_type(DataType::Character) // 70
            .push_byte_size(1)
            .push_character("i")
            .push_data_type(DataType::Character) // 73
            .push_byte_size(1)
            .push_character("t")
            .push_data_type(DataType::CharacterList) // 76
            .push_size(6) // 77
            .push_size(58) // 81
            .push_size(61) // 87
            .push_size(64) // 89
            .push_size(67) // 93
            .push_size(70) // 97
            .push_size(73) // 101
            .push_data_type(DataType::List) // 106
            .push_size(4)
            .push_size(2)
            .push_size(25)
            .push_size(44)
            .push_size(53)
            .push_size(76)
            .push_size(44) // keys
            .push_size(25);

        assert_eq!(result.data, expected);
    }

    #[test]
    fn writing_same_symbol_twice_only_has_one_additional_table_entry() {
        let mut result = vec![];
        let mut symbol_table = HashMap::new();

        ExpressionValue::symbol("my_symbol")
            .write_data(&mut result, &mut symbol_table)
            .unwrap();
        ExpressionValue::symbol("my_symbol")
            .write_data(&mut result, &mut symbol_table)
            .unwrap();

        assert_eq!(symbol_table.len(), 1);
        assert_eq!(*symbol_table.get("my_symbol").unwrap(), 0);
    }

    #[test]
    fn write() {
        let mut result = vec![];
        let mut symbol_table = HashMap::new();

        ExpressionValue::character_list("cats".into())
            .write_data(&mut result, &mut symbol_table)
            .unwrap();
        ExpressionValue::integer(100)
            .write_data(&mut result, &mut symbol_table)
            .unwrap();
        ExpressionValue::list()
            .add(ExpressionValue::pair(
                ExpressionValue::symbol("bear"),
                ExpressionValue::integer_range(Some(10), Some(20)).exclude_end(),
            ))
            .write_data(&mut result, &mut symbol_table)
            .unwrap();

        let mut expected = vec![];
        DataVecWriter::new(&mut expected)
            .push_data_type(DataType::Character) // 0
            .push_byte_size(1)
            .push_character("c")
            .push_data_type(DataType::Character) // 3
            .push_byte_size(1)
            .push_character("a")
            .push_data_type(DataType::Character) // 6
            .push_byte_size(1)
            .push_character("t")
            .push_data_type(DataType::Character) // 9
            .push_byte_size(1)
            .push_character("s")
            .push_data_type(DataType::CharacterList) // 12
            .push_size(4) // 13
            .push_size(0) // 17
            .push_size(3) // 21
            .push_size(6) // 25
            .push_size(9) // 29
            .push_data_type(DataType::Integer) // 33
            .push_integer(100)
            .push_data_type(DataType::Symbol) // 38
            .push_size(0)
            .push_data_type(DataType::Integer) // 41
            .push_integer(10)
            .push_data_type(DataType::Integer) // 48
            .push_integer(20)
            .push_data_type(DataType::Range) // 53
            .push_byte_size(RANGE_END_EXCLUSIVE)
            .push_integer(43)
            .push_integer(48)
            .push_data_type(DataType::Pair) // 63
            .push_size(38)
            .push_size(53)
            .push_data_type(DataType::List)
            .push_size(1)
            .push_size(1)
            .push_size(63)
            .push_size(63);

        assert_eq!(result, expected);
    }
}
