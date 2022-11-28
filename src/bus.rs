use crate::rom::*;

// memory map
//
// 0x0000 - 0x1FFF: RAM
// => 0x0000 - 0x00FF: ZeroPage
// => 0x0100 - 0x01FF: Stack
// => 0x0200 - 0x07FF: RAM
// => 0x0800 - 0x1FFF: Mirror
// 0x2000 - 0x3FFF: I/O
// => 0x2000 - 0x2007: PPU Registers
// 0x4000 - 0x5FFF: I/O & ExtROM
// => 0x4000 - 0x401F: I/O
// => 0x4020 - 0x5FFF: ExtROM
// 0x6000 - 0x7FFF: SRAM
// 0x8000 - 0xFFFF: ROM
// => 0xFFFC - 0xFFFD: Start Vector

const RAM: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU: u16 = 0x2000;
const PPU_END: u16 = 0x3FFF;
const ROM: u16 = 0x8000;
const ROM_END: u16 = 0xFFFF;

pub struct Bus {
    cpu_vram: [u8; 2048],
    rom: Rom,
}

pub trait Memory {
    fn memory_read_u8(&self, addr: u16) -> u8;
    fn memory_write_u8(&mut self, addr: u16, data: u8);
    fn memory_read_u16(&self, addr: u16) -> u16 {
        let lo = self.memory_read_u8(addr) as u16;
        let hi = self.memory_read_u8(addr + 1) as u16;
        return (hi << 8) | lo;
    }
    fn memory_write_u16(&mut self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00ff) as u8;
        self.memory_write_u8(addr, lo);
        self.memory_write_u8(addr + 1, hi);
    }
}

impl Bus {
    pub fn new_with_rom(rom: Rom) -> Self {
        Bus {
            cpu_vram: [0; 2048],
            rom: rom,
        }
    }

    pub fn new() -> Self {
        Bus {
            cpu_vram: [0; 2048],
            rom: Rom::new_null_rom().unwrap(),
        }
    }
}

impl Memory for Bus {
    fn memory_read_u8(&self, addr: u16) -> u8 {
        match addr {
            RAM ..= RAM_END => {
                let fixed_addr = addr & 0x07FF;
                return self.cpu_vram[fixed_addr as usize];
            },
            PPU ..= PPU_END => {
                // let fixed_addr = addr & 0x2007;
                // todo!("PPU is not supported yet.");
                0
            },
            ROM ..= ROM_END => {
                let mut fixed_addr = addr - 0x8000;
                if self.rom.prg_rom.len() == 0x4000 {
                    fixed_addr = fixed_addr & 0x3FFF;
                }
                return self.rom.prg_rom[fixed_addr as usize];
            },
            _ => {
                println!("invalid access at {:04x}",addr);
                return 0;
            }
        }
    }

    fn memory_write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            RAM ..= RAM_END => {
                let fixed_addr = addr & 0x07FF;
                self.cpu_vram[fixed_addr as usize] = data;
            },
            PPU ..= PPU_END => {
                // let fixed_addr = addr & 0x2007;
                // todo!("PPU is not supported yet.");
            },
            ROM ..= ROM_END => {
                println!("invalid write to ROM at {:04x}",addr);
            },
            _ => {
                println!("invalid access at {:04x}",addr);
            }
        }
    }

}