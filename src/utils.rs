pub fn to_u8(value: u16) -> (u8, u8) {
    // Converts u16 into (msb: u8, lsb: u8)
    let lsb = value & 0xff;
    let msb = (value & 0xff00) >> 8;
    (msb as u8, lsb as u8)
}

pub fn to_u16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

pub trait Bits {
    // Returns true if bit at index is set
    fn is_set(&self, index: u8) -> bool;
    fn is_unset(&self, index: u8) -> bool;
    // Returns 1 if bit at index is set
    fn get_bit(&self, index: u8) -> u8;
    fn set_bit(&self, index: u8) -> Self;
    fn unset_bit(&self, index: u8) -> Self;
}

impl Bits for u8 {
    fn is_set(&self, index: u8) -> bool {
        self & (1 << index) != 0
    }
    fn is_unset(&self, index: u8) -> bool {
        !self.is_set(index)
    }
    fn get_bit(&self, index: u8) -> u8 {
        self.is_set(index) as u8
    }
    fn set_bit(&self, index: u8) -> u8 {
        self | ((1 << index) as u8)
    }
    fn unset_bit(&self, index: u8) -> u8 {
        self & !((1 << index) as u8)
    }
}

impl Bits for u16 {
    fn is_set(&self, index: u8) -> bool {
        self & (1 << index) != 0
    }
    fn is_unset(&self, index: u8) -> bool {
        !self.is_set(index)
    }
    fn get_bit(&self, index: u8) -> u8 {
        self.is_set(index) as u8
    }
    fn set_bit(&self, index: u8) -> u16 {
        self | ((1 << index) as u16)
    }
    fn unset_bit(&self, index: u8) -> u16 {
        self & !((1 << index) as u16)
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

    #[test]
    fn set_bit_u8_1() {
        let val: u16 = 0b0100_0010;
        assert_eq!(val.set_bit(0), 0b0100_0011);
    }

    #[test]
    fn set_bit_u8_2() {
        let val: u16 = 0b0100_0010;
        assert_eq!(val.set_bit(1), 0b0100_0010);
    }

    #[test]
    fn set_bit_u8_3() {
        let val: u16 = 0x7F;
        assert_eq!(val.set_bit(7), 0xFF);
    }

    #[test]
    fn unset_bit_u8_1() {
        let val: u16 = 0b0100_0010;
        assert_eq!(val.unset_bit(1), 0b0100_0000);
    }

    #[test]
    fn unset_bit_u8_2() {
        let val: u16 = 0b0100_0010;
        assert_eq!(val.unset_bit(3), 0b0100_0010);
    }
}