use crate::{BasicData, BasicDataCustom, BasicGarnishData, DataError, basic::garnish::conversions::ConversionDelegate};

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

impl<'a, T> ConversionDelegate<T, char> for BasicDataDelegate<'a, T>
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

impl<'a, T> ConversionDelegate<T, char> for StringDelegate<'a, T>
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
    Delegate: ConversionDelegate<T, char, Output = Output>,
{
    delegate.init()?;
    convert_with_delegate(&mut delegate, from, 0)?;
    delegate.end()
}

fn convert_with_delegate<T>(delegate: &mut impl ConversionDelegate<T, char>, from: usize, depth: usize) -> Result<(), DataError>
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
            convert_with_delegate(delegate, start, depth + 1)?;
            delegate.push_char('.')?;
            delegate.push_char('.')?;
            convert_with_delegate(delegate, end, depth + 1)?;
        }
        BasicData::Slice(list, range) => {
            let (list, range) = (list.clone(), range.clone());

            if depth > 0 {
                delegate.push_char('(')?;
            }

            convert_with_delegate(delegate, list, depth + 1)?;
            delegate.push_char(' ')?;
            delegate.push_char('~')?;
            delegate.push_char(' ')?;
            convert_with_delegate(delegate, range, depth + 1)?;

            if depth > 0 {
                delegate.push_char(')')?;
            }
        }
        BasicData::Partial(reciever, input) => {
            let (reciever, input) = (reciever.clone(), input.clone());

            if depth > 0 {
                delegate.push_char('(')?;
            }

            convert_with_delegate(delegate, reciever, depth + 1)?;
            delegate.push_char(' ')?;
            delegate.push_char('~')?;
            delegate.push_char(' ')?;
            convert_with_delegate(delegate, input, depth + 1)?;

            if depth > 0 {
                delegate.push_char(')')?;
            }
        }
        BasicData::List(length, _) => {
            let end = from + 1 + length;
            let range = from + 1..end;

            if depth > 0 {
                delegate.push_char('(')?;
            }

            for i in range {
                let true_index = delegate.data().get_from_data_block_ensure_index(i)?.as_list_item()?;
                convert_with_delegate(delegate, true_index, depth + 1)?;

                if i < end - 1 {
                    delegate.push_char(' ')?;
                }
            }

            if depth > 0 {
                delegate.push_char(')')?;
            }
        }
        BasicData::Concatenation(left, right) => {
            let (left, right) = (left.clone(), right.clone());

            if depth > 0 {
                delegate.push_char('(')?;
            }

            convert_with_delegate(delegate, left, depth + 1)?;
            delegate.push_char(' ')?;
            delegate.push_char('<')?;
            delegate.push_char('>')?;
            delegate.push_char(' ')?;
            convert_with_delegate(delegate, right, depth + 1)?;

            if depth > 0 {
                delegate.push_char(')')?;
            }
        }
        BasicData::Custom(value) => {
            T::convert_custom_data_with_delegate(delegate, value.clone())?;
        }
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
mod tests {
    use garnish_lang_traits::{GarnishData, Instruction};

    use crate::{
        BasicData, BasicDataCustom, BasicGarnishData, ConversionDelegate, DataError, basic::{object::BasicObject, utilities::test_data}, basic_object
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
        unit: basic_object!(Unit) => "()",
        true_value: basic_object!(True) => "True",
        false_value: basic_object!(False) => "False",
        type_value: basic_object!(Type Number) => "Number",
        number: basic_object!(Number 100) => "100",
        char: basic_object!(Char 'a') => "a",
        byte: basic_object!(Byte 100) => "100",
        symbol: basic_object!(SymRaw 100) => "[Symbol 100]",
        symbol_with_string: basic_object!(Symbol "my_symbol") => ":my_symbol" with setup |data: &mut BasicGarnishData| {
            data.parse_add_symbol("my_symbol").unwrap();
        },
        symbol_list: basic_object!(SymList(SymRaw(100), Number 20)) => "[Symbol 100] 20",
        symbol_list_nested: basic_object!((SymList(SymRaw(100), Number 20)) = (SymList(SymRaw(200), Number 30))) => "([Symbol 100] 20) = ([Symbol 200] 30)",
        expression: basic_object!(Expression 123) => "[Expression 123]",
        external: basic_object!(External 123) => "[External 123]",
        char_list_clones: basic_object!(CharList "Formatted String") => "Formatted String",
        byte_list: basic_object!(ByteList 100, 200) => "100 200",
        byte_list_with_one_item: basic_object!(ByteList 100) => "100",
        byte_list_nested: basic_object!((ByteList 50) = (ByteList 100, 150, 200)) => "(50) = (100 150 200)",
        pair: basic_object!((CharList "value") = (Number 200)) => "value = 200",
        nested_pairs: basic_object!(((CharList "value") = (Number 200)) = ((CharList "value2") = (Number 400))) => "(value = 200) = (value2 = 400)",
        range: basic_object!((Number 100)..(Number 200)) => "100..200",
        slice: basic_object!((Number 100) - (Number 200)) => "100 ~ 200",
        partial: basic_object!((Number 100) ~ (Number 200)) => "100 ~ 200",
        list: basic_object!((Number 100), (Number 200)) => "100 200",
        concatenation: basic_object!((Number 100) <> (Number 200)) => "100 <> 200",
        list_concatenation_nested_under_pair: basic_object!(((Number 100) <> (Number 200)) = ((Number 300), (Number 400), (Number 500))) => "(100 <> 200) = (300 400 500)",
        slice_partial_under_pair: basic_object!(((Number 100) - (Number 200)) = ((Number 300) ~ (Number 500))) => "(100 ~ 200) = (300 ~ 500)",
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
    
    #[derive(Debug, Clone)]
    struct Foo {
        value: String,
    }

    impl BasicDataCustom for Foo {
        fn convert_custom_data_with_delegate(delegate: &mut impl ConversionDelegate<Self, char>, value: Self) -> Result<(), DataError> {
            delegate.push_char('F')?;
            delegate.push_char('o')?;
            delegate.push_char('o')?;
            delegate.push_char(' ')?;
            delegate.push_char('=')?;
            delegate.push_char(' ')?;
            for c in value.value.chars() {
                delegate.push_char(c)?;
            }

            Ok(())
        }
    }

    #[test]
    fn custom_data_converted() {
        let mut data = BasicGarnishData::<Foo>::new().unwrap();
        let index = data
            .push_object_to_data_block(basic_object!(Custom Foo {
                value: "custom value".to_string(),
            }))
            .unwrap();
        let char_list = data.convert_basic_data_at_to_char_list(index).unwrap();

        let length = data.get_from_data_block_ensure_index(char_list).unwrap().as_char_list().unwrap();

        let start = data.data_block.start + char_list + 1;
        let slice = &data.data[start..start + length];
        let result = slice.iter().map(|data| data.as_char().unwrap()).collect::<String>();
        assert_eq!(result, "Foo = custom value");

        let string = data.string_from_basic_data_at(index).unwrap();
        assert_eq!(string, "Foo = custom value");
    }
}
