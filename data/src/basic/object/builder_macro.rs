#[macro_export]
macro_rules! basic_object {
    (($($inner:tt)+)) => {
        basic_object!($($inner)+)
    };
    ($left:tt = $right:tt) => {
        $crate::BasicObject::Pair(Box::new(basic_object!($left)), Box::new(basic_object!($right)))
    };
    ($left:tt..$right:tt) => {
        $crate::BasicObject::Range(Box::new(basic_object!($left)), Box::new(basic_object!($right)))
    };
    ($left:tt - $right:tt) => {
        $crate::BasicObject::Slice(Box::new(basic_object!($left)), Box::new(basic_object!($right)))
    };
    ($left:tt ~ $right:tt) => {
        $crate::BasicObject::Partial(Box::new(basic_object!($left)), Box::new(basic_object!($right)))
    };
    ($left:tt <> $right:tt) => {
        $crate::BasicObject::Concatenation(Box::new(basic_object!($left)), Box::new(basic_object!($right)))
    };
    (Unit) => {
        $crate::BasicObject::Unit
    };
    (True) => {
        $crate::BasicObject::True
    };
    (False) => {
        $crate::BasicObject::False
    };
    (Type $garnish_type:ident) => {
        $crate::BasicObject::Type(::garnish_lang_traits::GarnishDataType::$garnish_type)
    };
    (Char $char_value:expr) => {
        $crate::BasicObject::Char($char_value)
    };
    (Byte $byte_value:expr) => {
        $crate::BasicObject::Byte($byte_value)
    };
    (Symbol $symbol_value:expr) => {
        $crate::BasicObject::Symbol($crate::symbol_value($symbol_value.trim_matches(':')))
    };
    (SymRaw $symbol_value:expr) => {
        $crate::BasicObject::Symbol($symbol_value)
    };
    (Number $value:expr) => {
        $crate::BasicObject::Number($value.into())
    };
    (External $value:expr) => {
        $crate::BasicObject::External($value)
    };
    (Expression $value:expr) => {
        $crate::BasicObject::Expression($value)
    };
    (CharList $value:expr) => {
        $crate::BasicObject::CharList($value.to_string())
    };
    (ByteList $($value:expr),*) => {
        $crate::BasicObject::ByteList(vec![
            $($value),*
        ])
    };
    (Custom $value:expr) => {
        $crate::BasicObject::Custom(Box::new($value))
    };
    (@symlist_part Symbol $value:expr) => {
        ::garnish_lang_traits::SymbolListPart::Symbol($crate::symbol_value($value.trim_matches(':')))
    };
    (@symlist_part SymRaw $value:expr) => {
        ::garnish_lang_traits::SymbolListPart::Symbol($value)
    };
    (@symlist_part Number $value:expr) => {
        ::garnish_lang_traits::SymbolListPart::Number($value.into())
    };
    // SymList pattern - handles both space-separated and parenthesized syntax
    (SymList()) => {
        $crate::BasicObject::SymbolList(vec![])
    };
    (SymList(SymRaw($first_val:expr) $(, $rest:tt $rest_val:expr)* $(,)?)) => {
        {
            let mut parts = vec![::garnish_lang_traits::SymbolListPart::Symbol($first_val)];
            $(
                parts.push(basic_object!(@symlist_part $rest $rest_val));
            )*
            $crate::BasicObject::SymbolList(parts)
        }
    };
    (SymList($first:tt $first_val:expr $(, $rest:tt $rest_val:expr)* $(,)?)) => {
        {
            let mut parts = vec![basic_object!(@symlist_part $first $first_val)];
            $(
                parts.push(basic_object!(@symlist_part $rest $rest_val));
            )*
            $crate::BasicObject::SymbolList(parts)
        }
    };
    ($($item:tt),+ $(,)?) => {
        $crate::BasicObject::List(vec![
            $(Box::new(basic_object!($item)),)*
        ])
    };
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::{GarnishDataType, SymbolListPart};

    use crate::basic::object::BasicObject;

    #[test]
    fn build_unit() {
        let value: BasicObject = basic_object!(Unit);

        assert_eq!(value, BasicObject::Unit);
    }

    #[test]
    fn build_false() {
        let value: BasicObject = basic_object!(False);

        assert_eq!(value, BasicObject::False);
    }

    #[test]
    fn build_true() {
        let value: BasicObject = basic_object!(True);

        assert_eq!(value, BasicObject::True);
    }

    #[test]
    fn garnish_type() {
        let value: BasicObject = basic_object!(Type Number);

        assert_eq!(value, BasicObject::Type(GarnishDataType::Number))
    }

    #[test]
    fn build_char() {
        let value: BasicObject = basic_object!(Char 'a');

        assert_eq!(value, BasicObject::Char('a'));
    }

    #[test]
    fn build_number() {
        let value: BasicObject = basic_object!(Number 100);

        assert_eq!(value, BasicObject::Number(100.into()));
    }

    #[test]
    fn build_byte() {
        let value: BasicObject = basic_object!(Byte 10);

        assert_eq!(value, BasicObject::Byte(10));
    }

    #[test]
    fn build_symbol() {
        let value: BasicObject = basic_object!(Symbol "my_symbol");

        assert_eq!(value, BasicObject::Symbol(8904929874702161741));
    }

    #[test]
    fn build_raw_symbol() {
        let value: BasicObject = basic_object!(SymRaw 12345);

        assert_eq!(value, BasicObject::Symbol(12345));
    }

    #[test]
    fn build_custom() {
        let value: BasicObject = basic_object!(Custom ());

        assert_eq!(value, BasicObject::Custom(Box::new(())));
    }

    #[test]
    fn build_external() {
        let value: BasicObject = basic_object!(External 10);

        assert_eq!(value, BasicObject::External(10));
    }

    #[test]
    fn build_expression() {
        let value: BasicObject = basic_object!(Expression 10);

        assert_eq!(value, BasicObject::Expression(10));
    }

    #[test]
    fn build_char_list() {
        let value: BasicObject = basic_object!(CharList "value");

        assert_eq!(value, BasicObject::CharList("value".to_string()));
    }

    #[test]
    fn build_byte_list() {
        let value: BasicObject = basic_object!(ByteList 100, 200, 250);

        assert_eq!(value, BasicObject::ByteList(vec![100, 200, 250]));
    }

    #[test]
    fn build_pair() {
        // Test simple values without parentheses
        let value1: BasicObject = basic_object!(Unit = True);
        assert_eq!(value1, BasicObject::Pair(Box::new(BasicObject::Unit), Box::new(BasicObject::True)));
        
        // Test Number = Number (requires parentheses for function-call syntax)
        let value: BasicObject = basic_object!((Number 100) = (Number 200));
        assert_eq!(value, BasicObject::Pair(Box::new(BasicObject::Number(100.into())), Box::new(BasicObject::Number(200.into()))));
        
        // Test different types on left and right
        let value2: BasicObject = basic_object!((Char 'a') = (Number 42));
        assert_eq!(value2, BasicObject::Pair(Box::new(BasicObject::Char('a')), Box::new(BasicObject::Number(42.into()))));
        
        // Test nested pairs (recursive) - each value in the inner pair needs parentheses
        let value3: BasicObject = basic_object!(((Number 10) = (Number 20)) = (Number 30));
        assert_eq!(value3, BasicObject::Pair(
            Box::new(BasicObject::Pair(
                Box::new(BasicObject::Number(10.into())),
                Box::new(BasicObject::Number(20.into()))
            )),
            Box::new(BasicObject::Number(30.into()))
        ));
        
        // Test pair with CharList
        let value4: BasicObject = basic_object!((CharList "hello") = (Number 42));
        assert_eq!(value4, BasicObject::Pair(
            Box::new(BasicObject::CharList("hello".to_string())),
            Box::new(BasicObject::Number(42.into()))
        ));
    }

    #[test]
    fn build_range() {
        let value: BasicObject = basic_object!((Number 10)..(Number 20));

        assert_eq!(value, BasicObject::Range(Box::new(BasicObject::Number(10.into())), Box::new(BasicObject::Number(20.into()))));
    }

    #[test]
    fn build_slice() {
        let value: BasicObject = basic_object!((Number 10) - (Number 20));

        assert_eq!(value, BasicObject::Slice(Box::new(BasicObject::Number(10.into())), Box::new(BasicObject::Number(20.into()))));
    }

    #[test]
    fn build_partial() {
        let value: BasicObject = basic_object!((Number 10) ~ (Number 20));

        assert_eq!(value, BasicObject::Partial(Box::new(BasicObject::Number(10.into())), Box::new(BasicObject::Number(20.into()))));
    }

    #[test]
    fn build_concatenation() {
        let value: BasicObject = basic_object!((Number 10) <> (Number 20));

        assert_eq!(value, BasicObject::Concatenation(Box::new(BasicObject::Number(10.into())), Box::new(BasicObject::Number(20.into()))));
    }

    #[test]
    fn build_list() {
        let value: BasicObject = basic_object!((Number 100), (Number 200), (Number 250));

        assert_eq!(value, BasicObject::List(vec![
            Box::new(BasicObject::Number(100.into())),
            Box::new(BasicObject::Number(200.into())),
            Box::new(BasicObject::Number(250.into()))
        ]));
    }

    #[test]
    fn build_symbol_list() {
        let value: BasicObject = basic_object!(SymList(Symbol "my_symbol", Number 100));

        assert_eq!(value, BasicObject::SymbolList(vec![
            SymbolListPart::Symbol(8904929874702161741),
            SymbolListPart::Number(100.into())
        ]));
    }

    #[test]
    fn build_list_with_byte_list() {
        let value: BasicObject = basic_object!((ByteList 100, 200, 250), (Number 100));

        assert_eq!(value, BasicObject::List(vec![
            Box::new(BasicObject::ByteList(vec![100, 200, 250])),
            Box::new(BasicObject::Number(100.into())),
        ]));
    }
}