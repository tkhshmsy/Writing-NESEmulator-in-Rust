use crate::ppu::NesPPU;
use crate::rom::Mirroring;

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

fn bg_palette(ppu: &NesPPU, attribute_table: &[u8], column: usize, row : usize) -> [u8; 4] {
    let attribute_table_index = row / 4 * 8 +  column / 4;
    let attribute_byte = attribute_table[attribute_table_index];

    let index = match (column % 4 / 2, row % 4 / 2) {
        (0, 0) => attribute_byte & 0x03,
        (1, 0) => (attribute_byte >> 2) & 0x03,
        (0, 1) => (attribute_byte >> 4) & 0x03,
        (1, 1) => (attribute_byte >> 6) & 0x03,
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

struct ViewRect {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl ViewRect {
    fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        ViewRect {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        }
    }
}

fn render_name_table(ppu: &NesPPU, frame: &mut Frame, name_table: &[u8],
    view_port: ViewRect, shift_x: isize, shift_y: isize) {
    let bank = ppu.control.background_pattern_address();
    let attribute_table = &name_table[0x3c0 .. 0x400];

    for i in 0 .. 0x3c0 {
        let index = name_table[i] as u16;
        let column = i % 32;
        let row = i / 32;
        let head = (bank + index* 16) as usize;
        let tail = (bank + index * 16 + 15) as usize;
        let tile = &ppu.chr_rom[head ..= tail];
        let palette = bg_palette(ppu, attribute_table, column, row);

        for y in 0 ..= 7 {
            let mut hi = tile[y];
            let mut lo = tile[y + 8];

            for x in (0 ..= 7).rev() {
                let value = (1 & lo) << 1 | (1 & hi);
                hi = hi >> 1;
                lo = lo >> 1;
                let rgb = match value {
                    0 => SYSTEM_PALETTE[ppu.palette_table[0] as usize],
                    1 => SYSTEM_PALETTE[palette[1] as usize],
                    2 => SYSTEM_PALETTE[palette[2] as usize],
                    3 => SYSTEM_PALETTE[palette[3] as usize],
                    _ => panic!("out of palette"),
                };
                let pixel_x = column * 8 + x;
                let pixel_y = row * 8 + y;

                if pixel_x >= view_port.x1 && pixel_x < view_port.x2 && pixel_y >= view_port.y1 && pixel_y < view_port.y2 {
                    frame.set_pixel((shift_x + pixel_x as isize) as usize, (shift_y + pixel_y as isize) as usize, rgb);
                }
            }
        }
    }
}

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let scroll_x = (ppu.scroll.scroll_x) as usize;
    let scroll_y = (ppu.scroll.scroll_y) as usize;
    let (main_name_table, second_name_table) = match (&ppu.mirroring, ppu.control.name_table_address()) {
        (Mirroring::VERTICAL, 0x2000)
        | (Mirroring::VERTICAL, 0x2800)
        | (Mirroring::HORIZONTAL, 0x2000)
        | (Mirroring::HORIZONTAL, 0x2400) => {
            (&ppu.vram[0 .. 0x400], &ppu.vram[0x400 .. 0x800])
        },
        (Mirroring::VERTICAL, 0x2400)
        | (Mirroring::VERTICAL, 0x2C00)
        | (Mirroring::HORIZONTAL, 0x2800)
        | (Mirroring::HORIZONTAL, 0x2C00) => {
            ( &ppu.vram[0x400 .. 0x800], &ppu.vram[0 .. 0x400])
        },
        (_, _) => {
            panic!("unsupported mirroring {:?}", ppu.mirroring);
        }
    };

    render_name_table(ppu, frame,
        main_name_table,
        ViewRect::new(scroll_x, scroll_y, 256, 240 ),
        -(scroll_x as isize), -(scroll_y as isize)
    );
    if scroll_x > 0 {
        render_name_table(ppu, frame,
            second_name_table,
            ViewRect::new(0, 0, scroll_x, 240),
            (256 - scroll_x) as isize, 0
        );
    } else if scroll_y > 0 {
        render_name_table(ppu, frame,
            second_name_table,
            ViewRect::new(0, 0, 256, scroll_y),
            0, (240 - scroll_y) as isize
        );
    }

    for i in (0 .. ppu.oam_data.len()).step_by(4).rev() {
        let index = ppu.oam_data[i + 1] as u16;
        let tx = ppu.oam_data[i + 3] as usize;
        let ty = ppu.oam_data[i] as usize;

        let flip_vertical = (((ppu.oam_data[i + 2] >> 7) & 0x01) == 0x01) as bool;
        let flip_horizontal = (((ppu.oam_data[i + 2] >> 6) & 0x01) == 0x01) as bool;
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
                    (false, false) => frame.set_pixel(tx     + x, ty     + y, rgb),
                    (true,  false) => frame.set_pixel(tx + 7 - x, ty     + y, rgb),
                    (false,  true) => frame.set_pixel(tx     + x, ty + 7 - y, rgb),
                    (true,   true) => frame.set_pixel(tx + 7 - x, ty + 7 - y, rgb),
                }
            }
        }
    }
}