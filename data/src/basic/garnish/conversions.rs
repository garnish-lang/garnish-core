use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError};

trait ConversionDelegate<T>
where
    T: BasicDataCustom,
{
    type Output;
    fn init(&mut self) -> Result<(), DataError>;
    fn push_char(&mut self, c: char) -> Result<(), DataError>;
    fn get_data_at(&self, index: usize) -> Result<&BasicData<T>, DataError>;
    fn end(self) -> Result<Self::Output, DataError>;
    fn data(&self) -> &BasicGarnishData<T>;
}

struct BasicDataDelegate<'a, T>
where
    T: BasicDataCustom,
{
    list_index: usize,
    length: usize,
    data: &'a mut BasicGarnishData<T>,
}

impl<'a, T> BasicDataDelegate<'a, T>
where
    T: BasicDataCustom,
{
    fn new(data: &'a mut BasicGarnishData<T>) -> Self {
        Self {
            list_index: 0,
            length: 0,
            data,
        }
    }
}

impl<'a, T> ConversionDelegate<T> for BasicDataDelegate<'a, T>
where
    T: BasicDataCustom,
{
    type Output = usize;

    fn init(&mut self) -> Result<(), DataError> {
        self.list_index = self.data.push_to_data_block(BasicData::CharList(0))?;
        Ok(())
    }

    fn push_char(&mut self, c: char) -> Result<(), DataError> {
        self.data.push_to_data_block(BasicData::Char(c))?;
        self.length += 1;
        Ok(())
    }

    fn get_data_at(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        self.data.get_from_data_block_ensure_index(index)
    }

    fn end(self) -> Result<Self::Output, DataError> {
        let list_length = self.data.get_from_data_block_ensure_index_mut(self.list_index)?.as_char_list_mut()?;
        *list_length = self.length;
        Ok(self.list_index)
    }

    fn data(&self) -> &BasicGarnishData<T> {
        self.data
    }
}

struct StringDelegate<'a, T>
where
    T: BasicDataCustom,
{
    string: String,
    data: &'a BasicGarnishData<T>,
}

impl<'a, T> StringDelegate<'a, T>
where
    T: BasicDataCustom,
{
    fn new(data: &'a BasicGarnishData<T>) -> Self {
        Self { string: String::new(), data }
    }
}

impl<'a, T> ConversionDelegate<T> for StringDelegate<'a, T>
where
    T: BasicDataCustom,
{
    type Output = String;

    fn init(&mut self) -> Result<(), DataError> {
        Ok(())
    }

    fn push_char(&mut self, c: char) -> Result<(), DataError> {
        self.string.push(c);
        Ok(())
    }

    fn get_data_at(&self, index: usize) -> Result<&BasicData<T>, DataError> {
        self.data.get_from_data_block_ensure_index(index)
    }

    fn end(self) -> Result<Self::Output, DataError> {
        Ok(self.string)
    }

    fn data(&self) -> &BasicGarnishData<T> {
        self.data
    }
}

impl<T> BasicGarnishData<T>
where
    T: BasicDataCustom,
{
    pub(crate) fn convert_basic_data_at_to_char_list(&mut self, from: usize) -> Result<usize, DataError> {
        init_convert_with_delegate(BasicDataDelegate::new(self), from)
    }

    pub(crate) fn string_from_basic_data_at(&self, from: usize) -> Result<String, DataError> {
        init_convert_with_delegate(StringDelegate::new(self), from)
    }
}

fn init_convert_with_delegate<Delegate, Output, T>(mut delegate: Delegate, from: usize) -> Result<Output, DataError>
where
    T: BasicDataCustom,
    Delegate: ConversionDelegate<T, Output = Output>,
{
    delegate.init()?;
    convert_with_delegate(&mut delegate, from, 0)?;
    delegate.end()
}

fn convert_with_delegate<T>(delegate: &mut impl ConversionDelegate<T>, from: usize, depth: usize) -> Result<(), DataError>
where
    T: BasicDataCustom,
{
    Ok(match delegate.get_data_at(from)? {
        BasicData::Unit => {
            delegate.push_char('(')?;
            delegate.push_char(')')?;
        }
        BasicData::True => {
            delegate.push_char('T')?;
            delegate.push_char('r')?;
            delegate.push_char('u')?;
            delegate.push_char('e')?;
        }
        BasicData::False => {
            delegate.push_char('F')?;
            delegate.push_char('a')?;
            delegate.push_char('l')?;
            delegate.push_char('s')?;
            delegate.push_char('e')?;
        }
        BasicData::Type(garnish_data_type) => {
            let s = format!("{}", garnish_data_type);
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::Number(simple_number) => {
            let s = simple_number.to_string();
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::Char(c) => {
            delegate.push_char(c.clone())?;
        }
        BasicData::Byte(b) => {
            let s = b.to_string();
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::Symbol(sym) => {
            match delegate.data().get_symbol_string(sym.clone())? {
                Some(s) => {
                    delegate.push_char(':')?;
                    for c in s.chars() {
                        delegate.push_char(c)?;
                    }
                    return Ok(());
                }
                None => {
                    let s = format!("[Symbol {}]", sym.to_string());
                    for c in s.chars() {
                        delegate.push_char(c)?;
                    }
                    return Ok(());
                }
            };
        }
        BasicData::SymbolList(length) => {
            let length = length.clone();
            let end = from + 1 + length;
            let range = from + 1..end;

            if depth > 0 {
                delegate.push_char('(')?;
            }

            for i in range {
                convert_with_delegate(delegate, i, depth + 1)?;

                if i < end - 1 {
                    delegate.push_char(' ')?;
                }
            }

            if depth > 0 {
                delegate.push_char(')')?;
            }
        }
        BasicData::Expression(jump_table_index) => {
            let s = format!("[Expression {}]", jump_table_index);
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::External(value) => {
            let s = format!("[External {}]", value);
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::ByteList(length) => {
            let length = length.clone();
            let end = from + 1 + length;
            let range = from + 1..end;

            if depth > 0 {
                delegate.push_char('(')?;
            }

            for i in range {
                convert_with_delegate(delegate, i, depth + 1)?;

                if i < end - 1 {
                    delegate.push_char(' ')?;
                }
            }

            if depth > 0 {
                delegate.push_char(')')?;
            }
        }
        BasicData::CharList(length) => {
            let start = from + 1;
            let length = length.clone();
            for i in start..start + length {
                let c = delegate.get_data_at(i)?.as_char()?;
                delegate.push_char(c)?;
            }
        }
        BasicData::Pair(left, right) => {
            let (left, right) = (left.clone(), right.clone());

            if depth > 0 {
                delegate.push_char('(')?;
            }

            convert_with_delegate(delegate, left, depth + 1)?;

            delegate.push_char(' ')?;
            delegate.push_char('=')?;
            delegate.push_char(' ')?;

            convert_with_delegate(delegate, right, depth + 1)?;

            if depth > 0 {
                delegate.push_char(')')?;
            }
        }
        BasicData::Range(start, end) => {
            let (start, end) = (start.clone(), end.clone());
            let start_s = delegate.data().string_from_basic_data_at(start)?;
            let end_s = delegate.data().string_from_basic_data_at(end)?;
            let s = format!("{}..{}", start_s, end_s);
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::Slice(list, range) => {
            let (list, range) = (list.clone(), range.clone());
            let list_s = delegate.data().string_from_basic_data_at(list)?;
            let range_s = delegate.data().string_from_basic_data_at(range)?;

            let s = format!("{} ~ {}", list_s, range_s);

            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::Partial(reciever, input) => {
            let (reciever, input) = (reciever.clone(), input.clone());
            let reciever_s = delegate.data().string_from_basic_data_at(reciever)?;
            let input_s = delegate.data().string_from_basic_data_at(input)?;
            let s = format!("{} ~ {}", reciever_s, input_s);
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::List(length, _) => {
            let mut strs = vec![];
            let range = from + 1..from + 1 + length;

            for i in range {
                let true_index = delegate.data().get_from_data_block_ensure_index(i)?.as_list_item()?;
                let s = delegate.data().string_from_basic_data_at(true_index)?;
                strs.push(s);
            }

            let s = strs.join(" ");
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::Concatenation(left, right) => {
            let (left, right) = (left.clone(), right.clone());
            let left_s = delegate.data().string_from_basic_data_at(left)?;
            let right_s = delegate.data().string_from_basic_data_at(right)?;
            let s = format!("{} <> {}", left_s, right_s);
            for c in s.chars() {
                delegate.push_char(c)?;
            }
        }
        BasicData::Custom(_) => todo!(),
        BasicData::Empty
        | BasicData::UninitializedList(_, _)
        | BasicData::ListItem(_)
        | BasicData::AssociativeItem(_, _)
        | BasicData::Value(_, _)
        | BasicData::Register(_, _)
        | BasicData::JumpPoint(_)
        | BasicData::Instruction(_, _)
        | BasicData::Frame(_, _) => {}
    })
}

#[cfg(test)]
mod convert_to_char_list {
    use garnish_lang_traits::{GarnishData, GarnishDataFactory, GarnishDataType, Instruction, SymbolListPart};

    use crate::{
        BasicData, BasicDataFactory, BasicGarnishData,
        basic::{object::BasicObject, utilities::test_data},
    };

    macro_rules! object_conversions {
        ($( $object_test_name:ident: $object:expr => $output:literal $(with setup $setup:expr)? ),+ $(,)?) => {
            $(#[test]
            fn $object_test_name() {
                let mut data = test_data();
                $($setup(&mut data);)?
                let index = data.push_object_to_data_block($object).unwrap();
                let char_list = data.convert_basic_data_at_to_char_list(index).unwrap();

                let length = data.get_from_data_block_ensure_index(char_list).unwrap().as_char_list().unwrap();

                let start = data.data_block.start + char_list + 1;
                let slice = &data.data[start..start + length];
                let result = slice.iter().map(|data| data.as_char().unwrap()).collect::<String>();
                assert_eq!(result, $output);

                let string = data.string_from_basic_data_at(index).unwrap();
                assert_eq!(string, $output);
            })*
        }
    }

    macro_rules! data_conversions {
        ($( $data_test_name:ident: $object:expr => $output:literal ),+ $(,)?) => {
            $(#[test]
            fn $data_test_name() {
                let mut data = test_data();
                let index = data.push_to_data_block($object).unwrap();
                let char_list = data.convert_basic_data_at_to_char_list(index).unwrap();

                let length = data.get_from_data_block_ensure_index(char_list).unwrap().as_char_list().unwrap();

                let start = char_list + 1;
                let slice = &data.data[start..start + length];
                let result = slice.iter().map(|data| data.as_char().unwrap()).collect::<String>();
                assert_eq!(result, $output);

                let string = data.string_from_basic_data_at(index).unwrap();
                assert_eq!(string, $output);
            })*
        }
    }

    object_conversions!(
        unit: BasicObject::Unit => "()",
        true_value: BasicObject::True => "True",
        false_value: BasicObject::False => "False",
        type_value: BasicObject::Type(GarnishDataType::Number) => "Number",
        number: BasicObject::Number(100.into()) => "100",
        char: BasicObject::Char('a') => "a",
        byte: BasicObject::Byte(100) => "100",
        symbol: BasicObject::Symbol(100) => "[Symbol 100]",
        symbol_with_string: BasicObject::Symbol(BasicDataFactory::parse_symbol("my_symbol").unwrap()) => ":my_symbol" with setup |data: &mut BasicGarnishData| {
            data.parse_add_symbol("my_symbol").unwrap();
        },
        symbol_list: BasicObject::SymbolList(vec![SymbolListPart::Symbol(100), SymbolListPart::Number(20.into())]) => "[Symbol 100] 20",
        symbol_list_nested: BasicObject::Pair(
            Box::new(BasicObject::SymbolList(vec![SymbolListPart::Symbol(100), SymbolListPart::Number(20.into())])),
            Box::new(BasicObject::SymbolList(vec![SymbolListPart::Symbol(200), SymbolListPart::Number(30.into())]))
        ) => "([Symbol 100] 20) = ([Symbol 200] 30)",
        expression: BasicObject::Expression(123) => "[Expression 123]",
        external: BasicObject::External(123) => "[External 123]",
        char_list_clones: BasicObject::CharList("Formatted String".to_string()) => "Formatted String",
        byte_list: BasicObject::ByteList(vec![100, 200]) => "100 200",
        byte_list_with_one_item: BasicObject::ByteList(vec![100]) => "100",
        byte_list_nested: BasicObject::Pair(Box::new(BasicObject::ByteList(vec![50])), Box::new(BasicObject::ByteList(vec![100, 150, 200]))) => "(50) = (100 150 200)",
        pair: BasicObject::Pair(Box::new(BasicObject::CharList("value".to_string())), Box::new(BasicObject::Number(200.into()))) => "value = 200",
        nested_pairs: BasicObject::Pair(
            Box::new(BasicObject::Pair(Box::new(BasicObject::CharList("value".to_string())), Box::new(BasicObject::Number(200.into())))),
            Box::new(BasicObject::Pair(Box::new(BasicObject::CharList("value2".to_string())), Box::new(BasicObject::Number(400.into()))))
        ) => "(value = 200) = (value2 = 400)",
        range: BasicObject::Range(Box::new(BasicObject::Number(100.into())), Box::new(BasicObject::Number(200.into()))) => "100..200",
        slice: BasicObject::Slice(Box::new(BasicObject::Number(100.into())), Box::new(BasicObject::Number(200.into()))) => "100 ~ 200",
        partial: BasicObject::Partial(Box::new(BasicObject::Number(100.into())), Box::new(BasicObject::Number(200.into()))) => "100 ~ 200",
        list: BasicObject::List(vec![Box::new(BasicObject::Number(100.into())), Box::new(BasicObject::Number(200.into()))]) => "100 200",
        concatenation: BasicObject::Concatenation(Box::new(BasicObject::Number(100.into())), Box::new(BasicObject::Number(200.into()))) => "100 <> 200",
        // custom: BasicObject::Custom(Box::new(BasicObject::Number(100.into()))) => "[Custom 100]",
    );

    data_conversions!(
        empty: BasicData::Empty => "",
        uninitialized_list: BasicData::UninitializedList(0, 0) => "",
        list_item: BasicData::ListItem(0) => "",
        associative_item: BasicData::AssociativeItem(0, 0) => "",
        value: BasicData::Value(None, 0) => "",
        register: BasicData::Register(None, 0) => "",
        instruction: BasicData::Instruction(Instruction::Add, None) => "",
        jump_point: BasicData::JumpPoint(0) => "",
        frame: BasicData::Frame(None, 0) => "",
    );
}
