use bitflags::*;
use std::collections::HashMap;
use crate::opcodes;

bitflags! {
    #[repr(transparent)]
    pub struct CpuFlags: u8 {
        const CARRY             = 0b0000_0001;
        const ZERO              = 0b0000_0010;
        const INTERRUPT_DISABLE = 0b0000_0100;
        const DECIMAL_MODE      = 0b0000_1000;
        const BREAK1            = 0b0001_0000;
        const BREAK2            = 0b0010_0000;
        const OVERFLOW          = 0b0100_0000;
        const NEGATIVE          = 0b1000_0000;
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NonAddressing,
}

pub struct CPU {
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_sp: u8,
    pub status: CpuFlags,
    pub reg_pc: u16,
    memory: [ u8; 0xFFFF ]
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_sp: 0,
            status: CpuFlags::empty(),
            reg_pc: 0,
            memory: [0; 0xFFFF],
        }
    }

    fn memory_read_u8(&self, addr: u16) -> u8 {
        return self.memory[addr as usize];
    }

    fn memory_write_u8(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn memory_read_u16(&self, addr: u16) -> u16 {
        let lo = self.memory_read_u8(addr) as u16;
        let hi = self.memory_read_u8(addr + 1) as u16;
        return (hi << 8) | (lo as u16);
    }

    fn memory_write_u16(&mut self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x0f) as u8;
        self.memory_write_u8(addr, lo);
        self.memory_write_u8(addr + 1, hi);
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.reg_pc,
            AddressingMode::ZeroPage => self.memory_read_u8(self.reg_pc) as u16,
            AddressingMode::Absolute => self.memory_read_u16(self.reg_pc),

            AddressingMode::ZeroPage_X => {
                let pos = self.memory_read_u8(self.reg_pc);
                let addr = pos.wrapping_add(self.reg_x) as u16;
                return addr;
            },
            AddressingMode::ZeroPage_Y  => {
                let pos = self.memory_read_u8(self.reg_pc);
                let addr = pos.wrapping_add(self.reg_y) as u16;
                return addr;
            },
            AddressingMode::Absolute_X => {
                let base = self.memory_read_u16(self.reg_pc);
                let addr = base.wrapping_add(self.reg_x as u16);
                return addr;
            },
            AddressingMode::Absolute_Y => {
                let base = self.memory_read_u16(self.reg_pc);
                let addr = base.wrapping_add(self.reg_y as u16);
                return addr;
            },
            AddressingMode::Indirect_X => {
                let base = self.memory_read_u8(self.reg_pc);
                let ptr = (base as u8).wrapping_add(self.reg_x);
                let lo = self.memory_read_u8(ptr as u16);
                let hi = self.memory_read_u8(ptr.wrapping_add(1) as u16);
                return (hi as u16) << 8 | (lo as u16)
            },
            AddressingMode::Indirect_Y => {
                let base = self.memory_read_u8(self.reg_pc);
                let lo = self.memory_read_u8(base as u16);
                let hi = self.memory_read_u8((base as u16).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.reg_y as u16);
                return deref;
            },
            AddressingMode::NonAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.memory_read_u8(addr);
        self.reg_a = value;
        self.update_cpuflags(self.reg_a);
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a;
        self.update_cpuflags(self.reg_x);
    }

    fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_cpuflags(self.reg_x);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        println!("sta addr:{:?}", addr);
        self.memory_write_u8(addr, self.reg_a);
    }

    fn update_cpuflags(&mut self, result: u8) {
        self.status.set(CpuFlags::ZERO, if result == 0 { true } else { false });
        self.status.set(CpuFlags::NEGATIVE, if result & 0b1000_0000 != 0 { true } else { false });
    }

    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.reg_sp = 0;
        self.status = CpuFlags::empty();
        self.reg_pc = self.memory_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.memory_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn run(&mut self) {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODE_MAP;

        loop {
            let code = self.memory_read_u8(self.reg_pc);
            self.reg_pc += 1;
            let pc_state = self.reg_pc;
            let opcode = opcodes.get(&code).expect(&format!("OpCode: {:?} is not recognized", code));

            match opcode.code {
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    // LDA
                    self.lda(&opcode.mode);
                },
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    // STA
                    self.sta(&opcode.mode);
                },
                0xaa => {
                    // TAX
                    self.tax();
                },
                0xe8 => {
                    // INX
                    self.inx();
                },
                0x00 => {
                    // BRK
                    return;
                },
                _ => todo!()
            }
            if pc_state == self.reg_pc {
                self.reg_pc += (opcode.len - 1) as u16;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.status.contains(CpuFlags::ZERO) == false);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE) == false);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.contains(CpuFlags::ZERO) == true);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
        assert_eq!(cpu.reg_x, 10);
    }

    #[test]
    fn test_0xe8_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 0xc1);
    }

    #[test]
    fn test_0xe8_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 1);
    }

    #[test]
    fn test_0xa5_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.memory_write_u8(0x10, 0x55);
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x55);
    }

    #[test]
    fn test_0x85_sta_to_zeropage() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x55, 0x85, 0x03, 0x00]);
        let data = cpu.memory_read_u8(0x03);
        assert_eq!(data, 0x55);
    }

    #[test]
    fn test_0x95_sta_to_zeropage_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x55, 0xaa, 0xa9, 0xaa, 0x95, 0x03, 0x00]);
        let data = cpu.memory_read_u8(0x58);
        assert_eq!(data, 0xaa);
    }
}

