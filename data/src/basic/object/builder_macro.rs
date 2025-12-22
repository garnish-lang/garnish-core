#[macro_export]
macro_rules! basic_object {
    (Unit) => {
        BasicObject::Unit
    };
    (True) => {
        BasicObject::True
    };
    (False) => {
        BasicObject::False
    };
    (Type($garnish_type:ident)) => {
        {
            use garnish_lang_traits::GarnishDataType;
            BasicObject::Type(GarnishDataType::$garnish_type)
        }
    };
    (Char($char_value:expr)) => {
        BasicObject::Char($char_value)
    };
    (Byte($byte_value:expr)) => {
        BasicObject::Byte($byte_value)
    };
    (Symbol($symbol_value:expr)) => {
        {
            use crate::basic::BasicDataFactory;
            use garnish_lang_traits::GarnishDataFactory;
            match BasicDataFactory::parse_symbol($symbol_value) {
                Ok(sym) => BasicObject::Symbol(sym),
                Err(_) => BasicObject::Symbol(0),
            }
        }
    };
    (SymRaw($symbol_value:expr)) => {
        BasicObject::Symbol($symbol_value)
    };
}

#[cfg(test)]
mod tests {
    use garnish_lang_traits::GarnishDataType;

    use crate::basic::object::BasicObject;

    use crate::basic_object;

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
        let value: BasicObject = basic_object!(Type(Number));

        assert_eq!(value, BasicObject::Type(GarnishDataType::Number))
    }

    #[test]
    fn build_char() {
        let value: BasicObject = basic_object!(Char('a'));

        assert_eq!(value, BasicObject::Char('a'));
    }

    #[test]
    fn build_byte() {
        let value: BasicObject = basic_object!(Byte(10));

        assert_eq!(value, BasicObject::Byte(10));
    }

    #[test]
    fn build_symbol() {
        let value: BasicObject = basic_object!(Symbol("my_symbol"));

        assert_eq!(value, BasicObject::Symbol(8904929874702161741));
    }

    #[test]
    fn build_raw_symbol() {
        let value: BasicObject = basic_object!(SymRaw(12345));

        assert_eq!(value, BasicObject::Symbol(12345));
    }
}