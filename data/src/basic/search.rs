use std::cmp::Ordering;

use crate::{BasicData, BasicDataCustom, DataError};

pub(crate) fn search_for_associative_item_index<T: BasicDataCustom>(items: &[BasicData<T>], search_symbol: u64) -> Result<Option<usize>, DataError> {
    let mut size = items.len();
    if size == 0 {
        return Ok(None);
    }
    
    let mut base = 0usize;
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        let cmp = items[mid].as_associative_item()?.0.cmp(&search_symbol);

        base = if cmp == Ordering::Greater { base } else { mid };

        size -= half;
    }

    let cmp = items[base].as_associative_item()?.0.cmp(&search_symbol);
    if cmp == Ordering::Equal {
        Ok(Some(base))
    } else {
        Ok(None)
    }
}

pub(crate) fn search_for_associative_item<T: BasicDataCustom>(items: &[BasicData<T>], search_symbol: u64) -> Result<Option<BasicData<T>>, DataError> {
    match search_for_associative_item_index(items, search_symbol)? {
        Some(index) => Ok(Some(items[index].clone())),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use crate::error::DataErrorType;

    use super::*;

    #[test]
    fn search_empty() {
        let items: Vec<BasicData<()>> = vec![];
        let result = search_for_associative_item(&items, 100);
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn search_ok_scenarios() {
        let items = vec![
            BasicData::<()>::AssociativeItem(100, 10),
            BasicData::<()>::AssociativeItem(200, 11),
            BasicData::<()>::AssociativeItem(300, 12),
            BasicData::<()>::AssociativeItem(400, 13),
            BasicData::<()>::AssociativeItem(500, 14),
            BasicData::<()>::AssociativeItem(600, 15),
            BasicData::<()>::AssociativeItem(700, 16),
            BasicData::<()>::AssociativeItem(800, 17),
            BasicData::<()>::AssociativeItem(900, 18),
            BasicData::<()>::AssociativeItem(1000, 19),
        ];

        let scenarios = vec![
            (100, Some(BasicData::<()>::AssociativeItem(100, 10))),
            (200, Some(BasicData::<()>::AssociativeItem(200, 11))),
            (300, Some(BasicData::<()>::AssociativeItem(300, 12))),
            (400, Some(BasicData::<()>::AssociativeItem(400, 13))),
            (500, Some(BasicData::<()>::AssociativeItem(500, 14))),
            (600, Some(BasicData::<()>::AssociativeItem(600, 15))),
            (700, Some(BasicData::<()>::AssociativeItem(700, 16))),
            (800, Some(BasicData::<()>::AssociativeItem(800, 17))),
            (900, Some(BasicData::<()>::AssociativeItem(900, 18))),
            (1000, Some(BasicData::<()>::AssociativeItem(1000, 19))),
            (50, None),
            (250, None),
            (750, None),
            (1250, None),
        ];

        for (sym, expected) in scenarios {
            let result = search_for_associative_item(&items, sym);
            assert_eq!(result, Ok(expected.clone()), "Expected {:?} for sym {} but got {:?}", expected, sym, result);
        }
    }

    #[test]
    fn search_not_ok() {
        let items = vec![
            BasicData::<()>::Unit,
            BasicData::<()>::Number(200.into()),
            BasicData::<()>::Symbol(300),
            BasicData::<()>::Expression(400),
            BasicData::<()>::External(500),
            BasicData::<()>::CharList(600),
            BasicData::<()>::ByteList(700),
            BasicData::<()>::Pair(800, 900),
            BasicData::<()>::Range(1000, 1100),
            BasicData::<()>::Slice(1200, 1300),
            BasicData::<()>::Partial(1400, 1500),
        ];

        let result = search_for_associative_item(&items, 100);
        assert_eq!(result, Err(DataError::new("Not a basic type", DataErrorType::NotBasicType)));
    }
}
