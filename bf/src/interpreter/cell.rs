use std::num::Wrapping;

#[derive(Clone, Debug, Default)]
pub struct Cell(Wrapping<u8>);

impl Cell {
    pub const fn new() -> Self {
        Self(Wrapping(0))
    }

    pub fn inc(&mut self) {
        self.0 += Wrapping(1);
    }

    pub fn dec(&mut self) {
        self.0 -= Wrapping(1);
    }

    pub const fn value(&self) -> u8 {
        (self.0).0
    }

    pub fn set(&mut self, value: u8) {
        self.0 = Wrapping(value);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_initializes_to_0() {
        let cell = Cell::new();
        assert_eq!(cell.0 .0, 0);
    }

    #[test]
    fn inc_increases_value_by_1() {
        let value = fastrand::u8(..u8::MAX);
        let mut cell = Cell(Wrapping(value));
        cell.inc();
        assert_eq!(cell.0 .0, value + 1);
    }

    #[test]
    fn dec_decreases_value_by_1() {
        let value = fastrand::u8(1..);
        let mut cell = Cell(Wrapping(value));
        cell.dec();
        assert_eq!(cell.0 .0, value - 1);
    }

    #[test]
    fn value_returns_the_value() {
        let value = fastrand::u8(..);
        let cell = Cell(Wrapping(value));
        assert_eq!(cell.value(), value);
    }

    #[test]
    fn set_updates_the_value() {
        let value = fastrand::u8(..);
        let mut cell = Cell::new();
        cell.set(value);
        assert_eq!(cell.0 .0, value);
    }
}
