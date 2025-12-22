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
}

#[cfg(test)]
mod tests {
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
}