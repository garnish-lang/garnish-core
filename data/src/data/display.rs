use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::data::{SimpleData, SimpleDataList};

pub trait DisplayForCustomItem {
    fn display_with_list(&self, list: &SimpleDataList<Self>, level: usize) -> String where Self: Clone + PartialEq + Eq + PartialOrd + Debug + Hash;
}

impl<T> SimpleData<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash + Display,
{
    pub fn display_simple(&self) -> String
    {
        match self {
            SimpleData::Unit => "()".into(),
            SimpleData::True => "True".into(),
            SimpleData::False => "False".into(),
            SimpleData::Type(t) => format!("Type({:?})", t),
            SimpleData::Number(n) => format!("{}", n),
            SimpleData::Char(c) => format!("{}", c),
            SimpleData::Byte(b) => b.to_string(),
            SimpleData::Symbol(s) => Self::display_simple_symbol(s),
            SimpleData::SymbolList(s) => format!("SymbolList({})", s.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", ")),
            SimpleData::Expression(e) => format!("Expression({})", e),
            SimpleData::External(e) => format!("External({})", e),
            SimpleData::CharList(s) => format!("{}", s),
            SimpleData::ByteList(l) => format!("'{}'", l.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(" ")),
            SimpleData::Pair(l, r) => format!("Pair({}, {})", l, r),
            SimpleData::Range(s, e) => format!("Range({}, {})", s, e),
            SimpleData::Slice(l, r) => format!("Slice({}, {})", l, r),
            SimpleData::Partial(l, r) => format!("Partial({}, {})", l, r),
            SimpleData::List(i, h) => format!(
                "List([{}], [{}])",
                i.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", "),
                h.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", ")
            ),
            SimpleData::Concatenation(l, r) => format!("Concatenation({}, {})", l, r),
            SimpleData::StackFrame(s) => format!("StackFrame{{return: {}}}", s.return_addr()),
            SimpleData::Custom(c) => format!("{}", c),
        }
    }

    pub fn display_simple_symbol(sym: &u64) -> String {
        format!("Symbol({})", sym)
    }
}

impl<T> SimpleDataList<T>
where
    T: Clone + PartialEq + Eq + PartialOrd + Debug + Hash + Display + DisplayForCustomItem,
{
    pub fn display_for_item(&self, index: usize) -> String
    {
        self.display_for_item_internal(index, 0)
    }

    fn display_for_item_internal(&self, index: usize, level: usize) -> String
    {
        match self.get(index) {
            None => String::from("<NoData>"),
            Some(item) => match item {
                SimpleData::Custom(c) => c.display_with_list(self, level),
                SimpleData::Symbol(s) => match self.symbol_to_name.get(s) {
                    None => item.display_simple(),
                    Some(s) => format!(":{}", s),
                },
                SimpleData::Expression(i) => match self.expression_to_symbol.get(i).and_then(|s| self.symbol_to_name.get(s)) {
                    None => item.display_simple(),
                    Some(s) => format!("Expression({})", s),
                },
                SimpleData::External(i) => match self.external_to_symbol.get(i).and_then(|s| self.symbol_to_name.get(s)) {
                    None => item.display_simple(),
                    Some(s) => format!("External({})", s),
                },
                SimpleData::ByteList(bytes) => match level > 0 {
                    true => format!("({})", bytes.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(" ")),
                    false => format!("{}", bytes.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(" ")),
                },
                SimpleData::SymbolList(symbols) => {
                    let base = symbols.iter().map(|s| match self.symbol_to_name.get(s) {
                        None => SimpleData::<T>::display_simple_symbol(s),
                        Some(s) => format!("{}", s),
                    }).collect::<Vec<String>>().join(".");

                    format!(":{}", base)
                }
                SimpleData::Pair(left, right) => {
                    format!(
                        "{} = {}",
                        self.display_for_item_internal(*left, level + 1),
                        self.display_for_item_internal(*right, level + 1)
                    )
                }
                SimpleData::Range(start, end) => format!(
                    "{}..{}",
                    self.display_for_item_internal(*start, level + 1),
                    self.display_for_item_internal(*end, level + 1)
                ),
                SimpleData::Concatenation(left, right) => {
                    let mut stack = vec![right, left];
                    let mut parts = vec![];

                    while let Some(item_index) = stack.pop() {
                        match self.get(*item_index) {
                            None => parts.push("<NoData>".to_string()),
                            Some(item) => match item {
                                SimpleData::Concatenation(left, right) => {
                                    stack.push(right);
                                    stack.push(left);
                                }
                                // concatenations of lists should appear to be a single list
                                // so we keep level the same for children
                                _ => parts.push(self.display_for_item_internal(*item_index, level)),
                            },
                        }
                    }

                    let base = parts.join(", ");

                    match level > 0 {
                        true => format!("({})", base),
                        false => base,
                    }
                }
                SimpleData::List(items, _) => {
                    let base = items
                        .iter()
                        .map(|item| self.display_for_item_internal(*item, level + 1))
                        .collect::<Vec<String>>()
                        .join(", ");

                    match level > 0 {
                        true => format!("({})", base),
                        false => base,
                    }
                }
                SimpleData::Slice(list, range) => {
                    let (start, end) = match self.get(*range) {
                        None => return format!("Slice({}, <NoData>)", list),
                        Some(item) => match item {
                            SimpleData::Range(start, end) => match (self.get(*start), self.get(*end)) {
                                (Some(SimpleData::Number(s)), Some(SimpleData::Number(e))) => {
                                    (s.to_integer().as_integer().unwrap(), e.to_integer().as_integer().unwrap())
                                }
                                (Some(SimpleData::Number(_s)), Some(_e)) => {
                                    return format!("Slice({}, {}..{}", list, start, self.display_for_item_internal(*end, level + 1))
                                }
                                (Some(_s), Some(SimpleData::Number(_e))) => {
                                    return format!("Slice({}, {}..{}", list, self.display_for_item_internal(*start, level + 1), end)
                                }
                                (Some(_s), Some(_e)) => {
                                    return format!(
                                        "Slice({}, ({})..({})",
                                        list,
                                        self.display_for_item_internal(*start, level + 1),
                                        self.display_for_item_internal(*end, level + 1)
                                    )
                                }
                                (Some(_s), None) => {
                                    return format!("Slice({}, ({})..<NoData>", list, self.display_for_item_internal(*start, level + 1))
                                }
                                (None, Some(_e)) => {
                                    return format!("Slice({}, <NoData>..({})", list, self.display_for_item_internal(*end, level + 1))
                                }
                                (None, None) => return format!("Slice({}, <NoData>..<NoData>", list),
                            },
                            _ => return format!("Slice({}, <NotARange>)", list),
                        },
                    };

                    // no support for negative numbers or reverse ranges yet
                    if start < 0 || end < 0 || start > end {
                        return String::from(format!("Slice({}, {}..{})", list, start, end));
                    }

                    let (start, end) = (start as usize, end as usize);
                    let count = end - start + 1;

                    match self.get(*list) {
                        None => format!("Slice(<NoData>, {}..{})", start, end),
                        Some(item) => match item {
                            SimpleData::List(items, _) => {
                                let base = items
                                    .iter()
                                    .skip(start)
                                    .take(count)
                                    .map(|item| self.display_for_item_internal(*item, level + 1))
                                    .collect::<Vec<String>>()
                                    .join(", ");

                                match level > 0 {
                                    true => format!("({})", base),
                                    false => base,
                                }
                            }
                            SimpleData::Concatenation(left, right) => {
                                let mut stack = vec![right, left];
                                let mut parts = vec![];

                                while let Some(item_index) = stack.pop() {
                                    match self.get(*item_index) {
                                        None => parts.push("<NoData>".to_string()),
                                        Some(item) => match item {
                                            SimpleData::Concatenation(left, right) => {
                                                stack.push(right);
                                                stack.push(left);
                                            }
                                            // flatten lists that are direct children of concatenation so it can be sliced
                                            SimpleData::List(items, _) => {
                                                for i in items {
                                                    parts.push(self.display_for_item_internal(*i, level + 1));
                                                }
                                            }
                                            // nested items should appear as though they are in list
                                            // so we keep level the same for children
                                            _ => parts.push(self.display_for_item_internal(*item_index, level)),
                                        },
                                    }
                                }

                                let base = parts.into_iter().skip(start).take(count).collect::<Vec<String>>().join(", ");

                                match level > 0 {
                                    true => format!("({})", base),
                                    false => base,
                                }
                            }
                            _ => format!("Slice({}, {}..{})", self.display_for_item_internal(*list, level + 1), start, end),
                        },
                    }
                }
                _ => item.display_simple(),
            },
        }
    }
}

#[cfg(test)]
mod shared {
    use std::fmt::{Display, Formatter};
    use crate::{DisplayForCustomItem, SimpleDataList};

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug, Hash)]
    pub struct Date {
        pub day: usize,
        pub month: usize,
    }

    impl Display for Date {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str(&format!("{}/{}", self.day, self.month))
        }
    }

    impl DisplayForCustomItem for Date {
        fn display_with_list(&self, _list: &SimpleDataList<Date>, _level: usize) -> String {
            format!("day {} of month {}", self.day, self.month)
        }
    }
}

#[cfg(test)]
mod simple {
    use garnish_lang_traits::GarnishDataType;

    use crate::data::display::shared::Date;
    use crate::data::{SimpleData, SimpleNumber};
    use crate::data::stack_frame::SimpleStackFrame;
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
        let data: SimpleData<NoCustom> = SimpleData::Type(GarnishDataType::Byte);
        assert_eq!(data.display_simple(), "Type(Byte)".to_string());
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
    fn simple_symbol_list() {
        let data: SimpleData<NoCustom> = SimpleData::SymbolList(vec![100, 200]);
        assert_eq!(data.display_simple(), "SymbolList(100, 200)".to_string());
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
        assert_eq!(data.display_simple(), "'10 20'".to_string());
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

    #[test]
    fn simple_stack_frame() {
        let data: SimpleData<NoCustom> = SimpleData::StackFrame(SimpleStackFrame::new(5));
        assert_eq!(data.display_simple(), "StackFrame{return: 5}".to_string());
    }

    #[test]
    fn simple_custom() {
        let data: SimpleData<Date> = SimpleData::Custom(Date { day: 1, month: 10 });
        assert_eq!(data.display_simple(), "1/10".to_string());
    }
}

#[cfg(test)]
mod simple_list {
    use garnish_lang_traits::GarnishDataType;

    use crate::data::display::shared::Date;
    use crate::data::{SimpleData, SimpleDataList, SimpleNumber};

    #[test]
    fn non_existent_item_is_none() {
        let list: SimpleDataList<Date> = SimpleDataList::new();

        assert_eq!(list.display_for_item(10), "<NoData>".to_string())
    }

    #[test]
    fn custom_data_formatter() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.push(SimpleData::Custom(Date { day: 1, month: 10 }));

        assert_eq!(list.display_for_item(0), "day 1 of month 10".to_string());
    }

    #[test]
    fn list_of_items() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.insert_symbol(150, "my_symbol");
        list.insert_symbol(250, "their_symbol");

        list.push(SimpleData::Unit);
        list.push(SimpleData::True);
        list.push(SimpleData::False);
        list.push(SimpleData::Type(GarnishDataType::Byte));
        list.push(SimpleData::Number(SimpleNumber::Integer(100)));
        list.push(SimpleData::Char('c'));
        list.push(SimpleData::Byte(10));
        list.push(SimpleData::Symbol(100));
        list.push(SimpleData::Expression(100));
        list.push(SimpleData::External(100));
        list.push(SimpleData::CharList("test".to_string()));
        list.push(SimpleData::ByteList(vec![10, 20]));
        list.push(SimpleData::Custom(Date { day: 1, month: 10 }));
        list.push(SimpleData::SymbolList(vec![150, 250]));

        list.push(SimpleData::List(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 100], vec![]));

        assert_eq!(
            list.display_for_item(14),
            "(), True, False, Type(Byte), 100, c, 10, Symbol(100), Expression(100), External(100), test, (10 20), day 1 of month 10, :my_symbol.their_symbol, <NoData>"
        );
    }

    #[test]
    fn symbol_name() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.insert_symbol(100, "my_symbol");
        list.push(SimpleData::Symbol(100));

        assert_eq!(list.display_for_item(0), ":my_symbol");
    }

    #[test]
    fn symbol_no_name() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();

        list.push(SimpleData::Symbol(100));

        assert_eq!(list.display_for_item(0), "Symbol(100)");
    }

    #[test]
    fn symbol_list() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.insert_symbol(100, "my_symbol");

        list.push(SimpleData::SymbolList(vec![100, 200]));

        assert_eq!(list.display_for_item(0), ":my_symbol.Symbol(200)".to_string());
    }

    #[test]
    fn expression_name() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.insert_symbol(100, "my_expression");
        list.insert_expression(1, 100);

        list.push(SimpleData::Expression(1));

        assert_eq!(list.display_for_item(0), "Expression(my_expression)");
    }

    #[test]
    fn expression_no_name() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.insert_symbol(100, "my_expression");

        list.push(SimpleData::Expression(1));

        assert_eq!(list.display_for_item(0), "Expression(1)");
    }

    #[test]
    fn external_name() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.insert_symbol(100, "my_external");
        list.insert_external(1, 100);

        list.push(SimpleData::External(1));

        assert_eq!(list.display_for_item(0), "External(my_external)");
    }

    #[test]
    fn external_no_name() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.insert_symbol(100, "my_external");

        list.push(SimpleData::External(1));

        assert_eq!(list.display_for_item(0), "External(1)");
    }

    #[test]
    fn pair() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.push(SimpleData::Number(SimpleNumber::Integer(100)));
        list.push(SimpleData::CharList("test".to_string()));

        list.push(SimpleData::Pair(0, 1));

        assert_eq!(list.display_for_item(2), "100 = test");
    }

    #[test]
    fn pair_nested() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.push(SimpleData::Number(SimpleNumber::Integer(100)));
        list.push(SimpleData::CharList("test".to_string()));
        list.push(SimpleData::Pair(0, 1));

        list.push(SimpleData::Number(SimpleNumber::Integer(200)));
        list.push(SimpleData::Pair(3, 2));

        assert_eq!(list.display_for_item(4), "200 = 100 = test");
    }

    #[test]
    fn range() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.push(SimpleData::Number(SimpleNumber::Integer(100)));
        list.push(SimpleData::Number(SimpleNumber::Integer(200)));

        list.push(SimpleData::Range(0, 1));

        assert_eq!(list.display_for_item(2), "100..200");
    }

    #[test]
    fn concatenation() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.push(SimpleData::Number(SimpleNumber::Integer(100)));
        list.push(SimpleData::Number(SimpleNumber::Integer(200)));
        list.push(SimpleData::Number(SimpleNumber::Integer(300)));
        list.push(SimpleData::Number(SimpleNumber::Integer(400)));

        list.push(SimpleData::Concatenation(0, 1));
        list.push(SimpleData::Concatenation(4, 2));
        list.push(SimpleData::Concatenation(3, 5));

        assert_eq!(list.display_for_item(6), "400, 100, 200, 300");
    }

    #[test]
    fn concatenation_of_list_with_concatenation_and_list() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.push(SimpleData::Number(SimpleNumber::Integer(100))); // 0
        list.push(SimpleData::Number(SimpleNumber::Integer(200))); // 1

        list.push(SimpleData::Concatenation(0, 1)); // 2
        list.push(SimpleData::Number(SimpleNumber::Integer(300))); // 3

        list.push(SimpleData::Number(SimpleNumber::Integer(400))); // 4

        list.push(SimpleData::Number(SimpleNumber::Integer(500))); // 5
        list.push(SimpleData::Number(SimpleNumber::Integer(600))); // 6
        list.push(SimpleData::Number(SimpleNumber::Integer(700))); // 7

        list.push(SimpleData::List(vec![5, 6, 7], vec![])); // 8

        list.push(SimpleData::List(vec![2, 3], vec![])); // 9
        list.push(SimpleData::List(vec![4, 8], vec![])); // 10

        list.push(SimpleData::Concatenation(9, 10));

        assert_eq!(list.display_for_item(11), "(100, 200), 300, 400, (500, 600, 700)");
    }

    #[test]
    fn slice_of_list() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.push(SimpleData::Number(SimpleNumber::Integer(100)));
        list.push(SimpleData::Number(SimpleNumber::Integer(200)));
        list.push(SimpleData::Number(SimpleNumber::Integer(300)));
        list.push(SimpleData::Number(SimpleNumber::Integer(400)));
        list.push(SimpleData::Number(SimpleNumber::Integer(500)));
        list.push(SimpleData::Number(SimpleNumber::Integer(600)));
        list.push(SimpleData::Number(SimpleNumber::Integer(700)));

        list.push(SimpleData::Number(SimpleNumber::Integer(2)));
        list.push(SimpleData::Number(SimpleNumber::Integer(5)));

        list.push(SimpleData::List(vec![0, 1, 2, 3, 4, 5, 6], vec![]));

        list.push(SimpleData::Range(7, 8));

        list.push(SimpleData::Slice(9, 10));

        assert_eq!(list.display_for_item(11), "300, 400, 500, 600");
    }

    #[test]
    fn slice_of_concatenation() {
        let mut list: SimpleDataList<Date> = SimpleDataList::new();
        list.push(SimpleData::Number(SimpleNumber::Integer(100)));
        list.push(SimpleData::Number(SimpleNumber::Integer(200)));
        list.push(SimpleData::Number(SimpleNumber::Integer(300)));
        list.push(SimpleData::Number(SimpleNumber::Integer(400)));
        list.push(SimpleData::Number(SimpleNumber::Integer(500)));
        list.push(SimpleData::Number(SimpleNumber::Integer(600)));
        list.push(SimpleData::Number(SimpleNumber::Integer(700)));

        list.push(SimpleData::Number(SimpleNumber::Integer(2)));
        list.push(SimpleData::Number(SimpleNumber::Integer(5)));

        list.push(SimpleData::List(vec![0, 1, 2, 3], vec![]));
        list.push(SimpleData::List(vec![4, 5, 6], vec![]));

        list.push(SimpleData::Concatenation(9, 10));

        list.push(SimpleData::Range(7, 8));

        list.push(SimpleData::Slice(11, 12));

        assert_eq!(list.display_for_item(13), "300, 400, 500, 600");
    }
}
