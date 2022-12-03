use bitflags::*;

bitflags! {
    // 7  bit  0
    // ---- ----
    // VPHB SINN
    // |||| ||||
    // |||| ||++- Base nametable address
    // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
    // |||| |     (0: add 1, going across; 1: add 32, going down)
    // |||| +---- Sprite pattern table address for 8x8 sprites
    // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
    // |||+------ Background pattern table address (0: $0000; 1: $1000)
    // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    // |+-------- PPU master/slave select
    // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
    // +--------- Generate an NMI at the start of the
    //            vertical blanking interval (0: off; 1: on)
    #[repr(transparent)]
    pub struct ControlRegister: u8 {
        const NAME_TABLE1                 = 0b0000_0001;
        const NAME_TABLE2                 = 0b0000_0010;
        const VRAM_ADD_INCREMENT          = 0b0000_0100;
        const SPRITE_PATTERN_ADDRESS      = 0b0000_1000;
        const BACKGROUND_PATTERN_ADDRESS  = 0b0001_0000;
        const SPRITE_SIZE                 = 0b0010_0000;
        const MASTER_SLAVE_SELECT         = 0b0100_0000;
        const GENERATE_NMI                = 0b1000_0000;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn vram_address_increment(&self) -> u8 {
        if self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            return 32;
        } else {
            return 1;
        }
    }

    pub fn name_table_address(&self) -> u16 {
        let value = self.bits & 0x03;
        match value {
            0x00 => { return 0x2000; },
            0x01 => { return 0x2400; },
            0x02 => { return 0x2800; },
            0x03 => { return 0x2c00; },
            _ => panic!("invalid name_table bits"),
        }
    }

    pub fn sprite_pattern_address(&self) -> u16 {
        if self.contains(ControlRegister::SPRITE_PATTERN_ADDRESS) {
            return 0x1000;
        } else {
            return 0;
        }
    }

    pub fn background_pattern_address(&self) -> u16 {
        if self.contains(ControlRegister::BACKGROUND_PATTERN_ADDRESS) {
            return 0x1000;
        } else {
            return 0;
        }
    }

    pub fn sprite_size(&self) -> u8 {
        if self.contains(ControlRegister::SPRITE_SIZE) {
            return 16;
        } else {
            return 8;
        }
    }

    pub fn master_slave_select(&self) -> bool {
        return self.contains(ControlRegister::MASTER_SLAVE_SELECT);
    }

    pub fn generate_vblank_nmi(&self) -> bool {
        return self.contains(ControlRegister::GENERATE_NMI);
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }
}