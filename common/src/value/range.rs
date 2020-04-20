pub const RANGE_START_EXCLUSIVE: u8 = 1;
pub const RANGE_END_EXCLUSIVE: u8 = 1 << 1;
pub const RANGE_OPEN_START: u8 = 1 << 2;
pub const RANGE_OPEN_END: u8 = 1 << 3;
pub const RANGE_HAS_STEP: u8 = 1 << 4;

pub fn has_step(flags: u8) -> bool {
    flags & RANGE_HAS_STEP == RANGE_HAS_STEP
}

pub fn has_start(flags: u8) -> bool {
    flags & RANGE_OPEN_START == 0
}

pub fn has_end(flags: u8) -> bool {
    flags & RANGE_OPEN_END == 0
}

pub fn is_end_exclusive(flags: u8) -> bool {
    flags & RANGE_END_EXCLUSIVE == RANGE_END_EXCLUSIVE
}

pub fn is_start_exclusive(flags: u8) -> bool {
    flags & RANGE_START_EXCLUSIVE == RANGE_START_EXCLUSIVE
}

pub fn empty_range_flags() -> u8 {
    0
}

pub trait RangeFlags {
    fn set_is_start_exclusive(self) -> Self;
    fn set_is_end_exclusive(self) -> Self;
    fn set_is_start_open(self) -> Self;
    fn set_is_end_open(self) -> Self;
    fn set_has_step(self) -> Self;
}

impl RangeFlags for u8 {
    fn set_is_start_exclusive(self) -> Self {
        self | RANGE_START_EXCLUSIVE
    }

    fn set_is_end_exclusive(self) -> Self {
        self | RANGE_END_EXCLUSIVE
    }

    fn set_is_start_open(self) -> Self {
        self | RANGE_OPEN_START
    }

    fn set_is_end_open(self) -> Self {
        self | RANGE_OPEN_END
    }

    fn set_has_step(self) -> Self {
        self | RANGE_HAS_STEP
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        has_end, has_start, has_step, is_end_exclusive, is_start_exclusive, RANGE_END_EXCLUSIVE,
        RANGE_HAS_STEP, RANGE_OPEN_END, RANGE_OPEN_START, RANGE_START_EXCLUSIVE,
    };

    #[test]
    fn has_step_is_true() {
        assert!(has_step(RANGE_HAS_STEP));
    }

    #[test]
    fn has_step_is_false() {
        assert!(!has_step(0));
    }

    #[test]
    fn has_start_is_true() {
        assert!(has_start(0));
    }

    #[test]
    fn has_start_is_false() {
        assert!(!has_start(RANGE_OPEN_START));
    }

    #[test]
    fn has_end_is_true() {
        assert!(has_end(0));
    }

    #[test]
    fn has_end_is_false() {
        assert!(!has_end(RANGE_OPEN_END));
    }

    #[test]
    fn exclusive_start_is_true() {
        assert!(is_start_exclusive(RANGE_START_EXCLUSIVE));
    }

    #[test]
    fn exclusive_start_is_false() {
        assert!(!is_start_exclusive(0));
    }

    #[test]
    fn exclusive_end_is_true() {
        assert!(is_end_exclusive(RANGE_END_EXCLUSIVE));
    }

    #[test]
    fn exclusive_end_is_false() {
        assert!(!is_end_exclusive(0));
    }

    // TODO: Figure out how to import trait methods here

    //    #[test]
    //    fn u8_set_is_exclusive_start() {
    //        let flags: u8 = 0;
    //        flags.set_is_exlusive_start();
    //
    //        assert!(is_exclusive_start(flags))
    //    }
    //
    //    #[test]
    //    fn u8_set_is_exclusive_end() {
    //        let flags: u8 = 0;
    //        flags.set_is_exlusive_end();
    //
    //        assert!(is_exclusive_end(flags))
    //    }
}
