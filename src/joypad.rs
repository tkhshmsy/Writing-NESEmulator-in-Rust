use bitflags::*;

bitflags!{
    pub struct JoypadButton: u8 {
        const RIGHT    = 0b1000_0000;
        const LEFT     = 0b0100_0000;
        const DOWN     = 0b0010_0000;
        const UP       = 0b0001_0000;
        const START    = 0b0000_1000;
        const SELECT   = 0b0000_0100;
        const BUTTON_B = 0b0000_0010;
        const BUTTON_A = 0b0000_0001;
    }
}

pub struct Joypad {
    strobe: bool,
    index: u8,
    status: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            index: 0,
            status: JoypadButton::from_bits_truncate(0x00),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = (data & 0x01) == 0x01;
        if self.strobe {
            self.index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.index > 7 {
            return 0x01;
        }
        let result = (self.status.bits & (1 << self.index)) >> self.index;
        if !self.strobe && self.index <= 7 {
            self.index += 1;
        }
        return result;
    }

    pub fn set_status(&mut self, button: JoypadButton, pressed: bool) {
        self.status.set(button, pressed);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_strobe() {
        let mut joypad = Joypad::new();
        joypad.write(1);
        joypad.set_status(JoypadButton::BUTTON_A, true);
        for _x in 0 .. 10 {
            assert_eq!(joypad.read(), 1);
        }
    }

    #[test]
    fn test_strobe_mode_flip() {
        let mut joypad = Joypad::new();

        joypad.write(0);
        joypad.set_status(JoypadButton::RIGHT,    true);
        joypad.set_status(JoypadButton::LEFT,     true);
        joypad.set_status(JoypadButton::SELECT,   true);
        joypad.set_status(JoypadButton::BUTTON_B, true);

        for _ in 0 ..= 1 {
            assert_eq!(joypad.read(), 0); // Btn A
            assert_eq!(joypad.read(), 1); // Btn B
            assert_eq!(joypad.read(), 1); // Select
            assert_eq!(joypad.read(), 0); // Start
            assert_eq!(joypad.read(), 0); // UP
            assert_eq!(joypad.read(), 0); // DOWN
            assert_eq!(joypad.read(), 1); // LEFT
            assert_eq!(joypad.read(), 1); // RIGHT

            for _x in 0 .. 10 {
                assert_eq!(joypad.read(), 1); // overrun
            }
            joypad.write(1);
            joypad.write(0);
        }
    }
}
