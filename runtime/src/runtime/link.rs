use crate::{next_two_raw_ref, ExpressionDataType, GarnishLangRuntimeData, RuntimeError};

pub(crate) fn append_link<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;
    link_internal(this,right, left, true)
}

pub(crate) fn prepend_link<Data: GarnishLangRuntimeData>(this: &mut Data) -> Result<(), RuntimeError<Data::Error>> {
    let (right, left) = next_two_raw_ref(this)?;
    link_internal(this, left, right, false)
}

pub fn link_internal<Data: GarnishLangRuntimeData>(this: &mut Data, value: Data::Size, link_to: Data::Size, is_append: bool) -> Result<(), RuntimeError<Data::Error>> {
    match this.get_data_type(link_to)? {
        ExpressionDataType::Link => {
            let value = match this.get_data_type(value)? {
                ExpressionDataType::Link => {
                    let (addr, ..) = this.get_link(value)?;
                    addr
                }
                _ => value
            };

            // create new link with value and link_to as linked
            let addr = this.add_link(value, link_to, is_append)?;
            this.push_register(addr)?;
        }
        _ => {
            let unit = this.add_unit()?;
            // unit is next value
            let linked = this.add_link(link_to, unit, is_append)?;
            // linked is next value
            let addr = this.add_link(value, linked, is_append)?;
            this.push_register(addr)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{runtime::GarnishRuntime, GarnishLangRuntimeData, SimpleRuntimeData, ExpressionDataType};

    #[test]
    fn append_link_create_new() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.append_link().unwrap();

        let (link1_value, link1_linked, is_append1) = runtime.get_link(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(link1_value).unwrap(), 20);
        assert_eq!(is_append1, true);

        let (link2_value, link2_linked, is_append2) = runtime.get_link(link1_linked).unwrap();
        assert_eq!(runtime.get_integer(link2_value).unwrap(), 10);
        assert_eq!(runtime.get_data_type(link2_linked).unwrap(), ExpressionDataType::Unit);
        assert_eq!(is_append2, true)
    }

    #[test]
    fn append_value_to_link() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();
        let d3 = runtime.add_link(d1, unit, true).unwrap();
        let d4 = runtime.add_link(d2, d3, true).unwrap();

        let d5 = runtime.add_integer(30).unwrap();

        runtime.push_register(d4).unwrap();
        runtime.push_register(d5).unwrap();

        runtime.append_link().unwrap();

        let (link1_value, link1_linked, is_append1) = runtime.get_link(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(link1_value).unwrap(), 30);
        assert_eq!(link1_linked, d4);
        assert_eq!(is_append1, true);
    }

    #[test]
    fn append_link_to_link() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();
        let d3 = runtime.add_link(d1, unit, true).unwrap();
        let d4 = runtime.add_link(d2, d3, true).unwrap();

        let d5 = runtime.add_integer(30).unwrap();
        let d6 = runtime.add_link(d5, unit, true).unwrap();

        runtime.push_register(d4).unwrap();
        runtime.push_register(d6).unwrap();

        runtime.append_link().unwrap();

        let (link1_value, link1_linked, is_append1) = runtime.get_link(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(link1_value).unwrap(), 30);
        assert_eq!(link1_linked, d4);
        assert_eq!(is_append1, true);
    }

    #[test]
    fn prepend_link_create_new() {
        let mut runtime = SimpleRuntimeData::new();

        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();

        runtime.push_register(d1).unwrap();
        runtime.push_register(d2).unwrap();

        runtime.prepend_link().unwrap();

        let (link1_value, link1_linked, is_append1) = runtime.get_link(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(link1_value).unwrap(), 10);
        assert_eq!(is_append1, false);

        let (link2_value, link2_linked, is_append2) = runtime.get_link(link1_linked).unwrap();
        assert_eq!(runtime.get_integer(link2_value).unwrap(), 20);
        assert_eq!(runtime.get_data_type(link2_linked).unwrap(), ExpressionDataType::Unit);
        assert_eq!(is_append2, false)
    }

    #[test]
    fn prepend_value_to_link() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();
        let d3 = runtime.add_link(d2, unit, false).unwrap();
        let d4 = runtime.add_link(d1, d3, false).unwrap();

        let d5 = runtime.add_integer(30).unwrap();

        runtime.push_register(d5).unwrap();
        runtime.push_register(d4).unwrap();

        runtime.prepend_link().unwrap();

        let (link1_value, link1_linked, is_append1) = runtime.get_link(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(link1_value).unwrap(), 30);
        assert_eq!(link1_linked, d4);
        assert_eq!(is_append1, false);
    }

    #[test]
    fn prepend_link_to_link() {
        let mut runtime = SimpleRuntimeData::new();

        let unit = runtime.add_unit().unwrap();
        let d1 = runtime.add_integer(10).unwrap();
        let d2 = runtime.add_integer(20).unwrap();
        let d3 = runtime.add_link(d2, unit, false).unwrap();
        let d4 = runtime.add_link(d1, d3, false).unwrap();

        let d5 = runtime.add_integer(30).unwrap();
        let d6 = runtime.add_link(d5, unit, false).unwrap();

        runtime.push_register(d6).unwrap();
        runtime.push_register(d4).unwrap();

        runtime.prepend_link().unwrap();

        let (link1_value, link1_linked, is_append1) = runtime.get_link(runtime.get_register(0).unwrap()).unwrap();
        assert_eq!(runtime.get_integer(link1_value).unwrap(), 30);
        assert_eq!(link1_linked, d4);
        assert_eq!(is_append1, false);
    }
}