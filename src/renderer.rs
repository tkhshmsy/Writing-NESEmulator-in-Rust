use crate::ppu::NesPPU;

#[rustfmt::skip]
pub static SYSTEM_PALETTE: [(u8,u8,u8); 64] = [
    (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            data: vec![0; (Frame::WIDTH) * (Frame::HEIGHT) * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * Frame::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
}

fn bg_palette(ppu: &NesPPU, column: usize, row : usize) -> [u8; 4] {
    let attr_table_index = row / 4 * 8 +  column / 4;
    let attr_byte = ppu.vram[0x03c0 + attr_table_index];

    let index = match (column % 4 / 2, row % 4 / 2) {
        (0, 0) => attr_byte & 0x03,
        (1, 0) => (attr_byte >> 2) & 0x03,
        (0, 1) => (attr_byte >> 4) & 0x03,
        (1, 1) => (attr_byte >> 6) & 0x03,
        (_, _) => panic!("invalid bg index"),
    };

    let palette_start: usize = 1 + (index as usize) * 4;
    return [ppu.palette_table[0],
            ppu.palette_table[palette_start],
            ppu.palette_table[palette_start + 1],
            ppu.palette_table[palette_start + 2]];
}

fn sprite_palette(ppu: &NesPPU, palette_index: u8) -> [u8; 4] {
    let start = 0x11 + (palette_index * 4) as usize;
    return [0,
            ppu.palette_table[start],
            ppu.palette_table[start + 1],
            ppu.palette_table[start + 2],
    ]
}

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let bank = ppu.control.background_pattern_address();
    for i in 0..0x03c0 {
        let ptr = ppu.vram[i] as u16;
        let tx = i % 32;
        let ty = i / 32;
        let head = (bank + ptr * 16) as usize;
        let tail = head + 15;
        let tile = &ppu.chr_rom[head..=tail];
        let palette = bg_palette(ppu, tx, ty);

        for y in 0..=7 {
            let mut hi = tile[y];
            let mut lo = tile[y + 8];
            for x in (0..=7).rev() {
                let value = (0x01 & lo) << 1 | (0x01 & hi);
                hi = hi >> 1;
                lo = lo >> 1;
                let rgb = match value {
                    0 => SYSTEM_PALETTE[ppu.palette_table[0] as usize],
                    1 => SYSTEM_PALETTE[palette[1] as usize],
                    2 => SYSTEM_PALETTE[palette[2] as usize],
                    3 => SYSTEM_PALETTE[palette[3] as usize],
                    _ => panic!("out of palette"),
                };
                frame.set_pixel(tx * 8 + x, ty * 8 + y, rgb);
            }
        }
    }

    for i in (0 .. ppu.oam_data.len()).step_by(4).rev() {
        let index = ppu.oam_data[i + 1] as u16;
        let tx = ppu.oam_data[i + 3] as usize;
        let ty = ppu.oam_data[i] as usize;

        let flip_vertical = ((ppu.oam_data[i + 2] >> 7) & 0x01 == 0x01) as bool;
        let flip_horizontal = ((ppu.oam_data[i + 2] >> 6) & 0x01 == 0x01) as bool;
        let palette_index = ppu.oam_data[i + 2] & 0x03;
        let sprite_palette = sprite_palette(ppu, palette_index);

        let bank: u16 = ppu.control.sprite_pattern_address();
        let head = (bank + index * 16) as usize;
        let tail = (bank + index * 16 + 15) as usize;
        let tile = &ppu.chr_rom[head ..= tail];

        for y in 0 ..= 7 {
            let mut hi = tile[y];
            let mut lo = tile[y + 8];
            'draw_sprite_row: for x in (0 ..= 7).rev() {
                let value = (0x01 & lo) << 1 | (0x01 & hi);
                hi = hi >> 1;
                lo = lo >> 1;
                let rgb = match value {
                    0 => continue 'draw_sprite_row,
                    1 => SYSTEM_PALETTE[sprite_palette[1] as usize],
                    2 => SYSTEM_PALETTE[sprite_palette[2] as usize],
                    3 => SYSTEM_PALETTE[sprite_palette[3] as usize],
                    _ => panic!("out of sprite palette"),
                };
                match (flip_horizontal, flip_vertical) {
                    (false, false) => frame.set_pixel(tx + x, ty + y, rgb),
                    (true,  false) => frame.set_pixel(tx + 7 - x, ty + y, rgb),
                    (false,  true) => frame.set_pixel(tx + x, ty + 7 - y, rgb),
                    (true,   true) => frame.set_pixel(tx + 7 - x, ty + 7 - y, rgb),
                }
            }
        }
    }
}