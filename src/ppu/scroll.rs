pub struct ScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub latch: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            latch: false,
        }
    }

    pub fn update(&mut self, data: u8) {
        if self.latch {
            self.scroll_y = data;
        } else {
            self.scroll_x = data;
        }
        self.latch = !self.latch;
    }

    pub fn reset(&mut self) {
        self.latch = false;
    }
}