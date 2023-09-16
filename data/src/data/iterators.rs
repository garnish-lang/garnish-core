use crate::data::number::SimpleNumber;
use garnish_traits::GarnishNumber;

pub struct SizeIterator {
    min: usize,
    max: usize,
    current_front: usize,
    current_back: usize,
}

impl SizeIterator {
    pub fn new(min: usize, max: usize) -> Self {
        Self {
            min,
            max,
            current_front: min,
            current_back: max,
        }
    }

    pub fn reset(&mut self) {
        self.current_front = self.min;
        self.current_back = self.max;
    }
}

impl Iterator for SizeIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_front >= self.current_back {
            return None;
        }

        self.current_front += 1;
        return Some(self.current_front - 1);
    }
}

impl DoubleEndedIterator for SizeIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current_back == 0 || self.current_back <= self.current_front {
            return None;
        }

        self.current_back -= 1;
        return Some(self.current_back);
    }
}

pub struct NumberIterator {
    min: SimpleNumber,
    max: SimpleNumber,
    current_front: SimpleNumber,
    current_back: SimpleNumber,
}

impl NumberIterator {
    pub fn new(min: SimpleNumber, max: SimpleNumber) -> Self {
        Self {
            min,
            max,
            current_front: min,
            current_back: max,
        }
    }

    pub fn reset(&mut self) {
        self.current_front = self.min;
        self.current_back = self.max;
    }
}

impl Iterator for NumberIterator {
    type Item = SimpleNumber;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_front >= self.current_back {
            return None;
        }

        self.current_front = self.current_front.plus(SimpleNumber::Integer(1)).unwrap_or(self.max);
        return Some(self.current_front.subtract(SimpleNumber::Integer(1)).unwrap_or(self.min));
    }
}

impl DoubleEndedIterator for NumberIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current_back == SimpleNumber::Integer(0) || self.current_back <= self.current_front {
            return None;
        }

        self.current_back = self.current_back.subtract(SimpleNumber::Integer(1)).unwrap_or(self.min);
        return Some(self.current_back);
    }
}

#[cfg(test)]
mod tests {
    use crate::data::SizeIterator;

    #[test]
    fn forward_iteration_full() {
        let mut iter = SizeIterator::new(0, 5);

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn backward_iteration_full() {
        let mut iter = SizeIterator::new(0, 5);

        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next_back(), Some(0));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn front_passes_back() {
        let mut iter = SizeIterator::new(0, 5);

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn back_passes_front() {
        let mut iter = SizeIterator::new(0, 5);

        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn reset() {
        let mut iter = SizeIterator::new(0, 5);

        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);

        iter.reset();

        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
