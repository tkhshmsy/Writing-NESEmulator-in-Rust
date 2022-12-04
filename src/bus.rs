use crate::rom::*;
use crate::ppu::*;
use crate::joypad::*;

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

const RAM: u16      = 0x0000;
const RAM_END: u16  = 0x1FFF;
const PPU: u16      = 0x2000;
const PPU_END: u16  = 0x3FFF;
const APU: u16      = 0x4000;
const APU_END: u16  = 0x4015;
const JOYPAD_1: u16 = 0x4016;
const JOYPAD_2: u16 = 0x4017;
const ROM: u16      = 0x8000;
const ROM_END: u16  = 0xFFFF;

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: NesPPU,
    joypad_1: Joypad,

    cycles: usize,
    vsync_callback: Box<dyn FnMut(&NesPPU, &mut Joypad) + 'call>,
}

pub trait Memory {
    fn memory_read_u8(&mut self, addr: u16) -> u8;
    fn memory_write_u8(&mut self, addr: u16, data: u8);
    fn memory_read_u16(&mut self, addr: u16) -> u16 {
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

impl<'a> Bus<'a> {
    pub fn new_with_rom<'call, F>(rom: Rom, vsync_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU, &mut Joypad) + 'call,
    {
        let ppu = NesPPU::new(rom.chr_rom, rom.screen_mirroring);
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu: ppu,
            joypad_1: Joypad::new(),
            cycles: 0,
            vsync_callback: Box::from(vsync_callback),
        }
    }

    pub fn new<'call, F>(vsync_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU, &mut Joypad) + 'call,
    {
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: [0; 16384].to_vec(),
            ppu: NesPPU::new_empty_rom(),
            joypad_1: Joypad::new(),
            cycles: 0,
            vsync_callback: Box::from(vsync_callback),
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles * 3); // PPU cycles are 3 times of CPU cycles
        let nmi_after = self.ppu.nmi_interrupt.is_some();

        if !nmi_before && nmi_after {
            (self.vsync_callback)(&self.ppu, &mut self.joypad_1);
        }
    }

    pub fn poll_nmi(&mut self) -> Option<u8> {
        return self.ppu.poll_nmi();
    }
}

impl Memory for Bus<'_> {
    fn memory_read_u8(&mut self, addr: u16) -> u8 {
        match addr {
            RAM ..= RAM_END => {
                let fixed_addr = addr & 0x07FF;
                return self.cpu_vram[fixed_addr as usize];
            },
            PPU_REG_CONTROLLER
            | PPU_REG_MASK
            | PPU_REG_OAM_ADDRESS
            | PPU_REG_SCROLL
            | PPU_REG_ADDRESS
            | PPU_REG_OAM_DMA => {
                // panic!("read write-only PPU register {:04x}", addr);
                return 0;
            },
            PPU_REG_STATUS => {
                return self.ppu.read_status();
            },
            PPU_REG_OAM_DATA => {
                return self.ppu.read_oam_data();
            },
            PPU_REG_DATA => {
                return self.ppu.read_data();
            },
            PPU_REG_END ..= PPU_END => {
                let fixed_addr = addr & 0x2007;
                return self.memory_read_u8(fixed_addr);
            },
            APU ..= APU_END => {
                // TODO
                return 0;
            },
            JOYPAD_1 => {
                return self.joypad_1.read();
            },
            JOYPAD_2 => {
                // TODO
                return 0;
            },
            ROM ..= ROM_END => {
                let mut fixed_addr = addr - 0x8000;
                if self.prg_rom.len() == 0x4000 {
                    fixed_addr = fixed_addr & 0x3FFF;
                }
                return self.prg_rom[fixed_addr as usize];
            },
            _ => {
                println!("invalid read at {:04x}",addr);
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
            PPU_REG_CONTROLLER => {
                self.ppu.write_control(data);
            },
            PPU_REG_MASK => {
                self.ppu.write_mask(data);
            },
            PPU_REG_STATUS => {
                panic!("invalid write to PPU Status register");
            },
            PPU_REG_OAM_ADDRESS => {
                self.ppu.write_oam_address(data);
            },
            PPU_REG_OAM_DATA => {
                self.ppu.write_oam_data(data);
            },
            PPU_REG_SCROLL => {
                self.ppu.write_scroll(data);
            },
            PPU_REG_ADDRESS => {
                self.ppu.write_address(data);
            },
            PPU_REG_DATA => {
                self.ppu.write_data(data);
            },
            PPU_REG_OAM_DMA => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi = (data as u16) << 8;
                for i in 0 .. 256u16 {
                    buffer[i as usize] = self.memory_read_u8(hi + i);
                }
                self.ppu.write_oam_dma(&buffer);
            },
            PPU_REG_END ..= PPU_END => {
                let fixed_addr = addr & 0x2007;
                self.memory_write_u8(fixed_addr, data);
            },
            APU ..= APU_END => {
                // TODO
            },
            JOYPAD_1 => {
                self.joypad_1.write(data);
            },
            JOYPAD_2 => {
                // TODO
            },
            ROM ..= ROM_END => {
                panic!("invalid write to ROM at {:04x}",addr);
            },
            _ => {
                println!("invalid write at {:04x}",addr);
            }
        }
    }

}