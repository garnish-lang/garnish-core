use crate::data::SimpleData;
use std::fmt::{Debug, Display};
use std::hash::Hash;

impl<T> SimpleData<T>
where
    T: Clone + Copy + PartialEq + Eq + PartialOrd + Debug + Hash,
{
    pub fn display_simple(&self) -> String where T: Display {
        match self {
            SimpleData::Unit => "()".into(),
            SimpleData::True => "True".into(),
            SimpleData::False => "False".into(),
            SimpleData::Type(t) => format!("{:?}", t),
            SimpleData::Number(n) => format!("{}", n),
            SimpleData::Char(c) => c.to_string(),
            SimpleData::Byte(b) => b.to_string(),
            SimpleData::Symbol(s) => format!("Symbol({})", s),
            SimpleData::Expression(e) => format!("Expression({})", e),
            SimpleData::External(e) => format!("External({})", e),
            SimpleData::CharList(s) => s.clone(),
            SimpleData::ByteList(l) => format!("{}", l.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(" ")),
            SimpleData::Pair(l, r) => format!("Pair({}, {})", l, r),
            SimpleData::Range(s, e) => format!("Range({}, {})", s, e),
            SimpleData::Slice(l, r) => format!("Slice({}, {})", l, r),
            SimpleData::List(i, h) => format!(
                "List([{}], [{}])",
                i.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", "),
                h.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", ")
            ),
            SimpleData::Concatenation(l, r) => format!("Concatenation({}, {})", l, r),
            SimpleData::Custom(c) => format!("{}", c),
        }
    }
}

#[cfg(test)]
mod simple {
    use std::fmt::{Display, Formatter};

    use garnish_traits::ExpressionDataType;

    use crate::data::{SimpleData, SimpleNumber};
    use crate::NoCustom;

    #[test]
    fn simple_unit() {
        let data: SimpleData<NoCustom> = SimpleData::Unit;
        assert_eq!(data.display_simple(), "()".to_string());
    }

    #[test]
    fn simple_true() {
        let data: SimpleData<NoCustom> = SimpleData::True;
        assert_eq!(data.display_simple(), "True".to_string());
    }

    #[test]
    fn simple_false() {
        let data: SimpleData<NoCustom> = SimpleData::False;
        assert_eq!(data.display_simple(), "False".to_string());
    }

    #[test]
    fn simple_type() {
        let data: SimpleData<NoCustom> = SimpleData::Type(ExpressionDataType::Byte);
        assert_eq!(data.display_simple(), "Byte".to_string());
    }

    #[test]
    fn simple_number() {
        let data: SimpleData<NoCustom> = SimpleData::Number(SimpleNumber::Integer(100));
        assert_eq!(data.display_simple(), "100".to_string());
    }

    #[test]
    fn simple_char() {
        let data: SimpleData<NoCustom> = SimpleData::Char('c');
        assert_eq!(data.display_simple(), "c".to_string());
    }

    #[test]
    fn simple_symbol() {
        let data: SimpleData<NoCustom> = SimpleData::Symbol(100);
        assert_eq!(data.display_simple(), "Symbol(100)".to_string());
    }

    #[test]
    fn simple_expression() {
        let data: SimpleData<NoCustom> = SimpleData::Expression(100);
        assert_eq!(data.display_simple(), "Expression(100)".to_string());
    }

    #[test]
    fn simple_external() {
        let data: SimpleData<NoCustom> = SimpleData::External(100);
        assert_eq!(data.display_simple(), "External(100)".to_string());
    }

    #[test]
    fn simple_character_list() {
        let data: SimpleData<NoCustom> = SimpleData::CharList("test".to_string());
        assert_eq!(data.display_simple(), "test".to_string());
    }

    #[test]
    fn simple_byte_list() {
        let data: SimpleData<NoCustom> = SimpleData::ByteList(vec![10, 20]);
        assert_eq!(data.display_simple(), "10 20".to_string());
    }

    #[test]
    fn simple_pair() {
        let data: SimpleData<NoCustom> = SimpleData::Pair(10, 20);
        assert_eq!(data.display_simple(), "Pair(10, 20)".to_string());
    }

    #[test]
    fn simple_range() {
        let data: SimpleData<NoCustom> = SimpleData::Range(10, 20);
        assert_eq!(data.display_simple(), "Range(10, 20)".to_string());
    }

    #[test]
    fn simple_slice() {
        let data: SimpleData<NoCustom> = SimpleData::Slice(10, 20);
        assert_eq!(data.display_simple(), "Slice(10, 20)".to_string());
    }

    #[test]
    fn simple_concatenation() {
        let data: SimpleData<NoCustom> = SimpleData::Concatenation(10, 20);
        assert_eq!(data.display_simple(), "Concatenation(10, 20)".to_string());
    }

    #[test]
    fn simple_list() {
        let data: SimpleData<NoCustom> = SimpleData::List(vec![10, 20], vec![20, 10]);
        assert_eq!(data.display_simple(), "List([10, 20], [20, 10])".to_string());
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug, Hash)]
    struct StructWith {}

    impl Display for StructWith {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str("StructWith")
        }
    }

    #[test]
    fn simple_custom() {
        let data: SimpleData<StructWith> = SimpleData::Custom(StructWith {});
        assert_eq!(data.display_simple(), "StructWith".to_string());
    }
}
