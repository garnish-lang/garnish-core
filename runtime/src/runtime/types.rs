
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum ExpressionDataType {
    Unit = 1,
    Integer,
    Float,
    Char,
    CharList,
    Byte,
    ByteList,
    Symbol,
    Pair,
    List,
    Expression,
    External,
    True,
    False,
}
