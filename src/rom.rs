// supported iNES 1.0 and mapper 0
use bitflags::*;

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOUR_SCREEN,
}

bitflags! {
    pub struct INesControlByte1: u8 {
        const VERTICAL_MIRRORING  = 0b0000_0001;
        const BATTERY_PACKED_RAM  = 0b0000_0010;
        const TRAINER             = 0b0000_0100;
        const FOUR_SCREEN_LAYOUT  = 0b0000_1000;
        const MAPPER_TYPE_LOW     = 0b1111_0000;
    }
}

bitflags! {
    pub struct INesControlByte2: u8 {
        const INES_VERSION_RESERVE = 0b0000_0011;
        const INES_VERSION         = 0b0000_1100;
        const MAPPER_TYPE_HIGHT    = 0b1111_0000;
    }
}

const NES_HEADER: usize = 0;
const NUM_16KB_ROM_BANKS: usize = 4;
const NUM_8KB_VROM_BANKS: usize = 5;
const NES_CONTROL_BYTE1: usize = 6;
const NES_CONTROL_BYTE2: usize = 7;
// const SIZE_PRG_RAM_8KB: usize = 8;

const NES_TAG: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Rom {
    pub fn new_empty_rom() -> Result<Rom, String> {
        return Ok (Rom {
            prg_rom: vec![0; 16384],
            chr_rom: vec![0; 16384],
            mapper: 0,
            screen_mirroring: Mirroring::VERTICAL,
        })
    }
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        if &raw[NES_HEADER .. NUM_16KB_ROM_BANKS] != NES_TAG {
            return Err("not iNES format".to_string());
        }
        let nes_control_byte1: INesControlByte1 = INesControlByte1::from_bits(raw[ NES_CONTROL_BYTE1 ]).unwrap();
        let nes_control_byte2: INesControlByte2 = INesControlByte2::from_bits(raw[ NES_CONTROL_BYTE2 ]).unwrap();
        let mapper = (nes_control_byte2 & INesControlByte2::MAPPER_TYPE_HIGHT).bits | ((nes_control_byte1 & INesControlByte1::MAPPER_TYPE_LOW).bits >> 4);
        let ines_ver = (nes_control_byte2 & INesControlByte2::INES_VERSION).bits >> 2;

        if ines_ver != 0 {
            return Err("not iNES 1.0".to_string());
        }

        let four_screen = nes_control_byte1.contains(INesControlByte1::FOUR_SCREEN_LAYOUT);
        let vertical_mirroring = nes_control_byte1.contains(INesControlByte1::VERTICAL_MIRRORING);
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOUR_SCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        let prg_rom_size = (raw[NUM_16KB_ROM_BANKS] as usize * PRG_ROM_PAGE_SIZE) as usize;
        let chr_rom_size = (raw[NUM_8KB_VROM_BANKS] as usize * CHR_ROM_PAGE_SIZE) as usize;
        let skip_trainer = nes_control_byte1.contains(INesControlByte1::TRAINER);
        let prg_rom_start = (16 + if skip_trainer { 512 } else { 0 }) as usize;
        let chr_rom_start = prg_rom_start + prg_rom_size;

        return Ok( Rom {
            prg_rom: raw[prg_rom_start .. (prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start .. (chr_rom_start + chr_rom_size)].to_vec(),
            mapper: mapper,
            screen_mirroring: screen_mirroring,
        });
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    struct TestRom {
        header: Vec<u8>,
        trainer: Option<Vec<u8>>,
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    }

    fn create_test_rom(rom: TestRom) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            rom.header.len() + rom.trainer.as_ref().map_or(0, |t| t.len())
                            + rom.prg_rom.len()
                            + rom.chr_rom.len(),
        );
        result.extend(&rom.header);
        if let Some(t) = rom.trainer {
            result.extend(t);
        }
        result.extend(&rom.prg_rom);
        result.extend(&rom.chr_rom);

        return result;
    }

    pub fn test_rom() -> Rom {
        let test_rom = create_test_rom(TestRom{
            header: vec![ 0x4e, 0x45, 0x53, 0x1a, 0x02, 0x01, 0x31, 0, 0, 0, 0, 0, 0, 0, 0, 0,],
            trainer: None,
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });
        return Rom::new(&test_rom).unwrap();
    }

    #[test]
    fn test() {
        let test_rom = create_test_rom(TestRom{
            header: vec![ 0x4e, 0x45, 0x53, 0x1a, 0x02, 0x01, 0x31, 0, 0, 0, 0, 0, 0, 0, 0, 0,],
            trainer: None,
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });
        let rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.prg_rom, vec![1; 2 * PRG_ROM_PAGE_SIZE]);
        assert_eq!(rom.chr_rom, vec![2; 1 * CHR_ROM_PAGE_SIZE]);
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::VERTICAL);
    }

    #[test]
    fn test_trainer() {
        let test_rom = create_test_rom(TestRom{
            header: vec![ 0x4e, 0x45, 0x53, 0x1a, 0x02, 0x01, (0x31 | 0x04), 0, 0, 0, 0, 0, 0, 0, 0, 0,],
            trainer: Some(vec![0; 512]),
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });
        let rom = Rom::new(&test_rom).unwrap();

        assert_eq!(rom.prg_rom, vec![1; 2 * PRG_ROM_PAGE_SIZE]);
        assert_eq!(rom.chr_rom, vec![2; 1 * CHR_ROM_PAGE_SIZE]);
        assert_eq!(rom.mapper, 3);
        assert_eq!(rom.screen_mirroring, Mirroring::VERTICAL);
    }

    #[test]
    fn test_unsupported_ines20() {
        let test_rom = create_test_rom(TestRom{
            header: vec![ 0x4e, 0x45, 0x53, 0x1a, 0x01, 0x01, 0x31, 8, 0, 0, 0, 0, 0, 0, 0, 0,],
            trainer: None,
            prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
        });
        let rom = Rom::new(&test_rom);
        match rom {
            Result::Ok(_) => assert!(false, "unexpected support ines2.0"),
            Result::Err(str) => assert_eq!(str, "not iNES 1.0"),
        }
    }

}