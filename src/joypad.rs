use crate::cpu::JOYPAD_INTERUPT;
use crate::utils::Bits;

pub const JOYPAD_REGISTER: u16 = 0xFF00;

// Index of bit indicating which buttons we are looking for
const DIRECTION_BIT: u8 = 4;
const BUTTON_BIT: u8 = 5;

#[derive(Clone, Copy)]
pub enum JoypadInput {
    Right = 0,
    Left = 1,
    Up = 2,
    Down = 3,
    A = 4,
    B = 5,
    Select = 6,
    Start = 7,
}

impl JoypadInput {
    pub fn is_button(&self) -> bool {
        !self.is_direction()
    }
    pub fn is_direction(&self) -> bool {
        (*self as u8) < JoypadInput::A as u8
    }
}

pub struct Joypad {
    input_pressed: [bool; 8],
    direction_selected: bool,
    button_selected: bool,
    pub int_request: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            input_pressed: [false; 8],
            direction_selected: false,
            button_selected: false,
            int_request: 0,
        }
    }

    pub fn on_key_pressed(&mut self, input: JoypadInput) {
        // Key was already pressed, nothing to do
        if self.input_pressed[input as usize] {
            return;
        }

        self.input_pressed[input as usize] = true;

        if (input.is_button() && self.button_selected)
            || (input.is_direction() && self.direction_selected)
        {
            self.int_request |= JOYPAD_INTERUPT;
        }
    }

    pub fn on_key_released(&mut self, input: JoypadInput) {
        self.input_pressed[input as usize] = false;
    }

    fn get_joypad_register(&self) -> u8 {
        // Translate our struct into the GB joypad register format

        // We set all bits and unset them
        let mut register = 0xFF;
        let mut shift = 0;

        if self.button_selected {
            register = register.unset_bit(BUTTON_BIT);
            shift = 4; // 4 upper values for self.input_pressed
        }

        if self.direction_selected {
            register = register.unset_bit(DIRECTION_BIT);
        }

        for i in 0..4 {
            if self.input_pressed[i + shift] {
                register = register.unset_bit(i as u8);
            }
        }

        register
    }

    pub fn readb(&self, addr: u16) -> u8 {
        match addr {
            JOYPAD_REGISTER => self.get_joypad_register(),
            _ => panic!("Unexpected read for joypad at {:x}", addr),
        }
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        match addr {
            JOYPAD_REGISTER => {
                self.direction_selected = value.is_unset(DIRECTION_BIT);
                self.button_selected = value.is_unset(BUTTON_BIT);
            }
            _ => panic!("Unexpected write for joypad at {:x}", addr),
        }
    }
}
