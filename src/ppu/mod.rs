use crate::rom::Mirroring;

const PPU_REG_CONTROLLER: u16  = 0x2000;
const PPU_REG_MASK: u16        = 0x2001;
const PPU_REG_STATUS: u16      = 0x2002;
const PPU_REG_OAM_ADDRESS: u16 = 0x2003;
const PPU_REG_OAM_DATA: u16    = 0x2004;
const PPU_REG_SCROLL: u16      = 0x2005;
const PPU_REG_ADDRESS: u16     = 0x2006;
const PPU_REG_DATA: u16        = 0x2007;
const PPU_REG_OAM_DMA: u16     = 0x4014;

const PPU_CHR_ROM: u16 = 0x0000;
const PPU_CHR_ROM_END: u16 = 0x1fff;
const PPU_VRAM: u16 = 0x2000;
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

    pub mirroring: Mirroring,
}

impl NesPPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom: chr_rom,
            mirroring: mirroring,

            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
        }
    }
} 
