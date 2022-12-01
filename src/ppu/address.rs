pub struct AddressRegister {
    value: (u8, u8), // hi, lo
    is_hi: bool,
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            value: (0, 0),
            is_hi: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = ((data & 0xff00) >> 8) as u8;
        self.value.1 = (data & 0x00ff) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.is_hi {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }
        if self.get() > 0x3fff {
            self.set(self.get() & 0x3fff);
        }
        self.is_hi = !self.is_hi;
    }

    pub fn get(&self) -> u16 {
        return ((self.value.0 as u16) << 8) | (self.value.1 as u16);
    }

    pub fn reset(&mut self) {
        self.is_hi = true;
    }

    pub fn increment(&mut self, value: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(value);
        if lo > self.value.1 { // overflow
            self.value.0 = self.value.0.wrapping_add(1);
        }
        if self.get() > 0x3fff {
            self.set(self.get() & 0x3fff);
        }
    }
}