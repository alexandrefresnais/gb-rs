pub trait Bits {
    // Returns true if bit at index is set
    fn is_set(&self, index: u8) -> bool;
    // Returns 1 if bit at index is set
    fn get_bit(&self, index: u8) -> u8;
}

impl Bits for u8 {
    fn is_set(&self, index: u8) -> bool {
        return self & (1 << index) != 0;
    }
    fn get_bit(&self, index: u8) -> u8 {
        self.is_set(index) as u8
    }
}

impl Bits for u16 {
    fn is_set(&self, index: u8) -> bool {
        return self & (1 << index) != 0;
    }
    fn get_bit(&self, index: u8) -> u8 {
        self.is_set(index) as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::Bits;

    #[test]
    fn bit_is_set_u8_1() {
        let val: u8 = 0xff;
        for i in 0..8 {
            assert_eq!(val.is_set(i), true);
        }
    }

    #[test]
    fn bit_is_set_u8_2() {
        let val: u8 = 0;
        for i in 0..8 {
            assert_eq!(val.is_set(i), false);
        }
    }

    #[test]
    fn bit_is_set_u8_3() {
        let val: u8 = 0b0100_0010;
        assert_eq!(val.is_set(0), false);
        assert_eq!(val.is_set(1), true);
        assert_eq!(val.is_set(2), false);
        assert_eq!(val.is_set(3), false);
        assert_eq!(val.is_set(4), false);
        assert_eq!(val.is_set(5), false);
        assert_eq!(val.is_set(6), true);
        assert_eq!(val.is_set(7), false);
    }

    #[test]
    fn bit_is_set_u16_1() {
        let val: u16 = 0xff;
        for i in 0..8 {
            assert_eq!(val.is_set(i), true);
        }
    }

    #[test]
    fn bit_is_set_u16_2() {
        let val: u16 = 0;
        for i in 0..8 {
            assert_eq!(val.is_set(i), false);
        }
    }


    #[test]
    fn bit_is_set_u16_3() {
        let val: u16 = 0b1001_0110_0100_0010;
        assert_eq!(val.is_set(0), false);
        assert_eq!(val.is_set(1), true);
        assert_eq!(val.is_set(2), false);
        assert_eq!(val.is_set(3), false);
        assert_eq!(val.is_set(4), false);
        assert_eq!(val.is_set(5), false);
        assert_eq!(val.is_set(6), true);
        assert_eq!(val.is_set(7), false);
        assert_eq!(val.is_set(8), false);
        assert_eq!(val.is_set(9), true);
        assert_eq!(val.is_set(10), true);
        assert_eq!(val.is_set(11), false);
        assert_eq!(val.is_set(12), true);
        assert_eq!(val.is_set(13), false);
        assert_eq!(val.is_set(14), false);
        assert_eq!(val.is_set(15), true);
    }
}