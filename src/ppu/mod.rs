pub mod address;
pub mod control;

use crate::rom::Mirroring;
use address::AddressRegister;
use control::ControlRegister;

pub const PPU_REG_CONTROLLER: u16  = 0x2000;
pub const PPU_REG_MASK: u16        = 0x2001;
pub const PPU_REG_STATUS: u16      = 0x2002;
pub const PPU_REG_OAM_ADDRESS: u16 = 0x2003;
pub const PPU_REG_OAM_DATA: u16    = 0x2004;
pub const PPU_REG_SCROLL: u16      = 0x2005;
pub const PPU_REG_ADDRESS: u16     = 0x2006;
pub const PPU_REG_DATA: u16        = 0x2007;
pub const PPU_REG_END: u16         = 0x2008;
pub const PPU_REG_OAM_DMA: u16     = 0x4014;

const PPU_CHR_ROM: u16 = 0x0000;
const PPU_CHR_ROM_END: u16 = 0x1fff;
const PPU_VRAM: u16 = 0x2000;
const PPU_VRAM_1ST_END: u16 = 0x2fff;
const PPU_VRAM_2ND: u16 = 0x3000;
const PPU_VRAM_END: u16 = 0x3eff;
const PPU_PALETTE_TABLE: u16 = 0x3f00;
const PPU_PALETTE_TABLE_END: u16 = 0x3fff;
const PPU_MIRRORS: u16 = 0x4000;
const PPU_MIRRORS_END: u16 = 0xffff;

pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    internal_data_buffer: u8,

    pub mirroring: Mirroring,
    pub address: AddressRegister,
    pub control: ControlRegister,
}

pub trait PPU {
    fn write_address(&mut self, data: u8);
    fn read_data(&mut self) -> u8;
    fn write_data(&mut self, data: u8);
    fn write_control(&mut self, data: u8);
}

impl NesPPU {
    pub fn new_empty_rom() -> Self {
        return NesPPU::new(vec![0; 2048], Mirroring::HORIZONTAL);
    }

    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom: chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
            internal_data_buffer: 0x00,

            mirroring: mirroring,
            address: AddressRegister::new(),
            control: ControlRegister::new(),
        }
    }

    fn increment_vram_address(&mut self) {
        self.address.increment(self.control.vram_address_increment());
    }

    fn mirror_vram_address(&mut self, addr: u16) -> u16 {
        let mirror_addr = addr & 0x2fff;
        let index = mirror_addr - PPU_VRAM;
        let table = index / 0x0400;
        match (&self.mirroring, table) {
            // HORIZONTAL -> AA'BB'
            (Mirroring::HORIZONTAL, 1) => index - 0x0400,
            (Mirroring::HORIZONTAL, 2) => index - 0x0400,
            (Mirroring::HORIZONTAL, 3) => index - 0x0800,
            // VERTICAL -> ABA'B'
            (Mirroring::VERTICAL, 2) => index - 0x0800,
            (Mirroring::VERTICAL, 3) => index - 0x0800,
            // FOUR_SCREEN -> ABCD
            _ => index,
        }
    }
}

impl PPU for NesPPU {
    fn write_address(&mut self, data: u8) {
        self.address.update(data);
    }

    fn read_data(&mut self) -> u8 {
        let addr = self.address.get();
        self.increment_vram_address();
        match addr {
            PPU_CHR_ROM ..= PPU_CHR_ROM_END => {
                let data = self.internal_data_buffer;
                self.internal_data_buffer = self.chr_rom[addr as usize];
                return data;
            },
            PPU_VRAM ..= PPU_VRAM_1ST_END => {
                let data = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_address(addr) as usize];
                return data;
            },
            PPU_VRAM_2ND ..= PPU_VRAM_END => {
                panic!("invalid PPU VRAM address {:04x}", addr);
            },
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let mirror_addr = addr - 0x10;
                return self.palette_table[(mirror_addr - PPU_PALETTE_TABLE) as usize];
            },
            PPU_PALETTE_TABLE ..= PPU_PALETTE_TABLE_END => {
                return self.palette_table[(addr - PPU_PALETTE_TABLE) as usize];
            },
            _ => panic!("invalid PPU address {:04x}", addr),
        }
    }

    fn write_data(&mut self, data: u8) {
        let addr = self.address.get();
        match addr {
            PPU_CHR_ROM ..= PPU_CHR_ROM_END => {
                println!("unable to write PPU CHR_ROM for {:04x}", addr);
            },
            PPU_VRAM ..= PPU_VRAM_1ST_END => {
                self.vram[self.mirror_vram_address(addr) as usize] = data;
            },
            PPU_VRAM_2ND ..= PPU_VRAM_END => {
                panic!("invalid PPU VRAM address {:04x}", addr);
            },
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let mirror_addr = addr - 0x10;
                self.palette_table[(mirror_addr - PPU_PALETTE_TABLE) as usize] = data;
            },
            PPU_PALETTE_TABLE ..= PPU_PALETTE_TABLE_END => {
                self.palette_table[(addr - PPU_PALETTE_TABLE) as usize] = data;
            },
            _ => panic!("invalid PPU address {:04x}", addr),
        }
        self.increment_vram_address();
    }

    fn write_control(&mut self, data: u8) {
        self.control.update(data);
    }
} 

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_ppu_write_vram() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_address(0x23);
        ppu.write_address(0x05);
        ppu.write_data(0x66);
        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_read_vram() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_control(0);
        ppu.vram[0x0305] = 0x66;
        ppu.write_address(0x23);
        ppu.write_address(0x05);
        ppu.read_data(); // read, then address+=1
        assert_eq!(ppu.address.get(), 0x2306);
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_read_over_page() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_control(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77; // across page

        ppu.write_address(0x21);
        ppu.write_address(0xff);

        ppu.read_data();
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_control(ControlRegister::VRAM_ADD_INCREMENT.bits());
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.write_address(0x21);
        ppu.write_address(0xff);

        ppu.read_data();
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
        assert_eq!(ppu.read_data(), 0x88);
    }

    #[test]
    fn test_vram_horizontal_mirror() {
        let mut ppu = NesPPU::new(vec![0; 2048], Mirroring::HORIZONTAL);
        // HORIZONTAL -> AA'BB'
        ppu.write_address(0x24);
        ppu.write_address(0x05);
        ppu.write_data(0x66); //write to A'
        ppu.write_address(0x28);
        ppu.write_address(0x05);
        ppu.write_data(0x77); //write to B

        ppu.write_address(0x20);
        ppu.write_address(0x05);
        ppu.read_data();
        assert_eq!(ppu.read_data(), 0x66); // read A' from A

        ppu.write_address(0x2C);
        ppu.write_address(0x05);
        ppu.read_data();
        assert_eq!(ppu.read_data(), 0x77); //read B from B'
    }

    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = NesPPU::new(vec![0; 2048], Mirroring::VERTICAL);
        // VERTICAL -> ABA'B'
        ppu.write_address(0x20);
        ppu.write_address(0x05);
        ppu.write_data(0x66); //write to A
        ppu.write_address(0x2C);
        ppu.write_address(0x05);
        ppu.write_data(0x77); //write to B'

        ppu.write_address(0x28);
        ppu.write_address(0x05);
        ppu.read_data();
        assert_eq!(ppu.read_data(), 0x66); //read A from A'

        ppu.write_address(0x24);
        ppu.write_address(0x05);
        ppu.read_data();
        assert_eq!(ppu.read_data(), 0x77); //read B' from B
    }
}