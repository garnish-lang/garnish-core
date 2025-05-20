use crate::runtime::utilities::next_two_raw_ref;
use garnish_lang_traits::{GarnishData, RuntimeError};

pub fn partial_apply<Data: GarnishData>(this: &mut Data) -> Result<Option<Data::Size>, RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;

    this.add_partial(left, right).and_then(|i| this.push_register(i))?;

    Ok(None)
}

#[cfg(test)]
mod tests {
    use crate::runtime::partial::partial_apply;
    use crate::runtime::tests::MockGarnishData;
    use garnish_lang_traits::GarnishDataType;

    #[test]
    fn test_partial_apply() {
        let mut data = MockGarnishData::new_basic_data(vec![GarnishDataType::Expression, GarnishDataType::List]);

        data.stub_add_partial = |_, left, right| {
            assert_eq!(0, left);
            assert_eq!(1, right);
            Ok(100)
        };
        data.stub_push_register = |_, addr| {
            assert_eq!(100, addr);
            Ok(())
        };

        let result = partial_apply(&mut data).unwrap();
        
        assert_eq!(result, None);
    }
}
