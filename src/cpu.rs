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

// memory map
const STACK_BASE: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

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
            reg_sp: STACK_RESET,
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

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.memory_read_u8(addr);
        let result = self.reg_a.overflowing_add(value);
        self.status.set(CpuFlags::CARRY, result.1);
        self.status.set(CpuFlags::OVERFLOW, (result.0 ^ value) & (result.0 ^ self.reg_a) & 0x80 != 0x00);
        self.reg_a = result.0;
        self.update_cpuflags(self.reg_a);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.memory_read_u8(addr);
        self.reg_a = self.reg_a & value;
        self.update_cpuflags(self.reg_a);
    }

    fn asl_accumulator(&mut self) {
        let value = self.reg_a;
        self.status.set(CpuFlags::CARRY, value & 0x80 == 0x80);
        self.reg_a = value << 1;
        self.update_cpuflags(self.reg_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.memory_read_u8(addr);
        self.status.set(CpuFlags::CARRY, value & 0x80 == 0x80);
        value = value << 1;
        self.memory_write_u8(addr, value);
        self.update_cpuflags(value);
    }

    fn bcc(&mut self) {
        if !self.status.contains(CpuFlags::CARRY) {
            self.branch();
        }
    }

    fn bcs(&mut self) {
        if self.status.contains(CpuFlags::CARRY) {
            self.branch();
        }
    }

    fn beq(&mut self) {
        if self.status.contains(CpuFlags::ZERO) {
            self.branch();
        }
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.memory_read_u8(addr);
        self.status.set(CpuFlags::ZERO, self.reg_a & value == 0x00);
        self.status.set(CpuFlags::OVERFLOW, value & 0x40 == 0x40);
        self.status.set(CpuFlags::NEGATIVE, value & 0x80 == 0x80);
    }

    fn bmi(&mut self) {
        if self.status.contains(CpuFlags::NEGATIVE) {
            self.branch();
        }
    }

    fn bne(&mut self) {
        if !self.status.contains(CpuFlags::ZERO) {
            self.branch();
        }
    }

    fn bpl(&mut self) {
        if !self.status.contains(CpuFlags::NEGATIVE) {
            self.branch();
        }
    }

    fn bvc(&mut self) {
        if !self.status.contains(CpuFlags::OVERFLOW) {
            self.branch();
        }
    }

    fn bvs(&mut self) {
        if self.status.contains(CpuFlags::OVERFLOW) {
            self.branch();
        }
    }

    fn clc(&mut self) {
        self.status.set(CpuFlags::CARRY, false);
    }

    fn cld(&mut self) {
        self.status.set(CpuFlags::DECIMAL_MODE, false);
    }

    fn cli(&mut self) {
        self.status.set(CpuFlags::INTERRUPT_DISABLE, false);
    }

    fn clv(&mut self) {
        self.status.set(CpuFlags::OVERFLOW, false);
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value =  self.memory_read_u8(addr);
        self.compare(self.reg_a, value);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let value =  self.memory_read_u8(addr);
        self.compare(self.reg_x, value);
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let value =  self.memory_read_u8(addr);
        self.compare(self.reg_y, value);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let mut value =  self.memory_read_u8(addr);
        value = value.wrapping_sub(1);
        self.memory_write_u8(addr, value);
        self.update_cpuflags(value);
    }

    fn dex(&mut self) {
        self.reg_x = self.reg_x.wrapping_sub(1);
        self.update_cpuflags(self.reg_x);
    }

    fn dey(&mut self) {
        self.reg_y = self.reg_y.wrapping_sub(1);
        self.update_cpuflags(self.reg_y);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let value =  self.memory_read_u8(addr);
        self.reg_a = self.reg_a ^ value;
        self.update_cpuflags(self.reg_a);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let mut value =  self.memory_read_u8(addr);
        value = value.wrapping_add(1);
        self.memory_write_u8(addr, value);
        self.update_cpuflags(value);
    }

    fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_cpuflags(self.reg_x);
    }

    fn iny(&mut self) {
        self.reg_y = self.reg_y.wrapping_add(1);
        self.update_cpuflags(self.reg_y);
    }

    fn jmp(&mut self) {
        let addr =  self.memory_read_u16(self.reg_pc);
        self.reg_pc = addr;
    }

    fn jmp_indirect(&mut self) {
        let addr =  self.memory_read_u16(self.reg_pc);
        let indirect = if addr & 0x00ff == 0x00ff {
            // reproduction 6502 bug
            let lo = self.memory_read_u8(addr);
            let hi = self.memory_read_u8(addr & 0xff00);
            (hi as u16) << 8  | (lo as u16)
        } else {
            self.memory_read_u16(addr)
        };
        self.reg_pc = indirect;
    }

    fn jsr(&mut self) {
        // TODO
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.memory_read_u8(addr);
        self.reg_a = value;
        self.update_cpuflags(self.reg_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.memory_read_u8(addr);
        self.reg_x = value;
        self.update_cpuflags(self.reg_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.memory_read_u8(addr);
        self.reg_y = value;
        self.update_cpuflags(self.reg_y);
    }

    fn lsr_accumulator(&mut self) {
        let value = self.reg_a;
        self.status.set(CpuFlags::CARRY, value & 0x01 == 0x01);
        self.reg_a = value >> 1;
        self.update_cpuflags(self.reg_a);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.memory_read_u8(addr);
        self.status.set(CpuFlags::CARRY, value & 0x01 == 0x01);
        value = value >> 1;
        self.memory_write_u8(addr, value);
        self.update_cpuflags(value);
    }

    fn nop(&mut self) {
        return;
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let value =  self.memory_read_u8(addr);
        self.reg_a = self.reg_a | value;
        self.update_cpuflags(self.reg_a);
    }

    fn pha(&mut self) {
        self.stack_push_u8(self.reg_a);
    }

    fn php(&mut self) {
        let flags = self.status.bits();
        self.stack_push_u8(flags);
    }

    fn pla(&mut self) {
        self.reg_a = self.stack_pop_u8();
        self.update_cpuflags(self.reg_a);
    }

    fn plp(&mut self) {
        let flags = self.stack_pop_u8();
        self.status.bits = flags;
    }

    fn rol_accumulator(&mut self) {
        let value = self.reg_a;
        let old_carry = self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY, value & 0x80 == 0x80);
        self.reg_a = value << 1;
        if old_carry {
            self.reg_a |= 0x01;
        }
        self.update_cpuflags(self.reg_a);
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.memory_read_u8(addr);
        let old_carry = self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY, value & 0x80== 0x80);
        value = value << 1;
        if old_carry {
            value |= 0x01;
        }
        self.memory_write_u8(addr, value);
        self.update_cpuflags(value);
    }

    fn ror_accumulator(&mut self) {
        let value = self.reg_a;
        let old_carry = self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY, value & 0x01 == 0x01);
        self.reg_a = value >> 1;
        if old_carry {
            self.reg_a |= 0x80;
        }
        self.update_cpuflags(self.reg_a);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.memory_read_u8(addr);
        let old_carry = self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY, value & 0x01 == 0x01);
        value = value >> 1;
        if old_carry {
            value |= 0x80;
        }
        self.memory_write_u8(addr, value);
        self.update_cpuflags(value);
    }

    fn rti(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn rts(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn sec(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn sed(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn sei(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.memory_write_u8(addr, self.reg_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn sty(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a;
        self.update_cpuflags(self.reg_x);
    }

    fn tay(&mut self) {
        self.reg_y = self.reg_a;
        self.update_cpuflags(self.reg_y);
    }

    fn tsx(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn txa(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn txs(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn tya(&mut self, mode: &AddressingMode) {
        // TODO
    }

    fn update_cpuflags(&mut self, data: u8) {
        self.status.set(CpuFlags::ZERO, data == 0);
        self.status.set(CpuFlags::NEGATIVE, data & 0b1000_0000 != 0);
    }

    fn branch(&mut self) {
        let dst = self.memory_read_u8(self.reg_pc) as i8;
        let addr = self.reg_pc.wrapping_add(1).wrapping_add(dst as u16);
        self.reg_pc = addr;
    }

    fn compare(&mut self, lhs: u8, rhs: u8) {
        self.status.set(CpuFlags::CARRY, lhs >= rhs);
        self.update_cpuflags(lhs.wrapping_sub(rhs));
    }

    fn stack_push_u8(&mut self, data: u8) {
        self.memory_write_u8((STACK_BASE as u16) + (self.reg_sp as u16), data);
        self.reg_sp = self.reg_sp.wrapping_sub(1);
    }

    fn stack_push_u16(&mut self, data: u16) {
        let lo = (data & 0x00ff) as u8;
        let hi = (data >> 8) as u8;
        self.stack_push_u8(hi);
        self.stack_push_u8(lo);
    }

    fn stack_pop_u8(&mut self) -> u8 {
        self.reg_sp = self.reg_sp.wrapping_add(1);
        return self.memory_read_u8((STACK_BASE as u16) + (self.reg_sp as u16));
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop_u8() as u16;
        let hi = self.stack_pop_u8() as u16;
        return hi << 8 | lo;
    }

    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.reg_sp = STACK_RESET;
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
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => {
                    // ADC
                    self.adc(&opcode.mode);
                },
                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => {
                    // AND
                    self.and(&opcode.mode);
                },
                0x0a => {
                    // ASL Accumulator
                    self.asl_accumulator();
                },
                0x06 | 0x16 | 0x0e | 0x1e => {
                    // ASL
                    self.asl(&opcode.mode);
                },
                0x90 => {
                    // BCC
                    self.bcc();
                },
                0xb0 => {
                    // BCS
                    self.bcs();
                },
                0xf0 => {
                    // BEQ
                    self.beq();
                },
                0x24 | 0x2c => {
                    // BIT
                    self.bit(&opcode.mode);
                },
                0x30 => {
                    // BMI
                    self.bmi();
                },
                0xd0 => {
                    // BNE
                    self.bne();
                },
                0x10 => {
                    // BPL
                    self.bpl();
                },
                0x00 => {
                    // BRK
                    return;
                },
                0x50 => {
                    // BVC
                    self.bvc();
                },
                0x70 => {
                    // BVS
                    self.bvs();
                },
                0x18 => {
                    // CLC
                    self.clc();
                },
                0xd8 => {
                    // CLD
                    self.cld();
                },
                0x58 => {
                    // CLI
                    self.cli();
                },
                0xb8 => {
                    // CLV
                    self.clv();
                },
                0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => {
                    // CMP
                    self.cmp(&opcode.mode);
                },
                0xe0 | 0xe4 | 0xec => {
                    // CPX
                    self.cpx(&opcode.mode);
                },
                0xc0 | 0xc4 | 0xcc => {
                    // CPY
                    self.cpy(&opcode.mode);
                },
                0xc6 | 0xd6 | 0xce | 0xde => {
                    // DEC
                    self.dec(&opcode.mode);
                },
                0xca => {
                    // DEX
                    self.dex();
                },
                0x88 => {
                    // DEY
                    self.dey();
                },
                0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => {
                    // EOR
                    self.eor(&opcode.mode);
                },
                0xe6 | 0xf6 | 0xee | 0xfe => {
                    // INC
                    self.inc(&opcode.mode);
                },
                0xe8 => {
                    // INX
                    self.inx();
                },
                0xc8 => {
                    // INY
                    self.iny();
                },
                0x4c => {
                    // JMP
                    self.jmp();
                },
                0x6c => {
                    // JMP indirect
                    self.jmp_indirect();
                },
                // TODO
                // 0x20 => {
                //     // JSR
                //     self.jsr();
                // },
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    // LDA
                    self.lda(&opcode.mode);
                },
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => {
                    // LDX
                    self.ldx(&opcode.mode);
                },
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => {
                    // LDY
                    self.ldy(&opcode.mode);
                },
                0x4a => {
                    // LSR Accumulator
                    self.lsr_accumulator();
                },
                0x46 | 0x56 | 0x4e | 0x5e => {
                    // LSR
                    self.lsr(&opcode.mode);
                },
                0xea => {
                    // NOP
                    self.nop();
                },
                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => {
                    // ORA
                    self.ora(&opcode.mode);
                },
                0x48 => {
                    // PHA
                    self.pha();
                },
                0x08 => {
                    // PHP
                    self.php();
                },
                0x68 => {
                    // PLA
                    self.pla();
                },
                0x28 => {
                    // PHP
                    self.plp();
                },
                0x2a => {
                    // ROL Accumulator
                    self.rol_accumulator();
                },
                0x26 | 0x36 | 0x2e | 0x3e => {
                    // ROL
                    self.rol(&opcode.mode);
                },
                0x6a => {
                    // ROR Accumulator
                    self.ror_accumulator();
                },
                0x66 | 0x76 | 0x6e | 0x7e => {
                    // ROR
                    self.ror(&opcode.mode);
                },





                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    // STA
                    self.sta(&opcode.mode);
                },
                0xaa => {
                    // TAX
                    self.tax();
                },
                0xa8 => {
                    // TAY
                    self.tay();
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
    fn test_0x69_adc_immidiate_for_no_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x55, 0x69, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x65);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x69_adc_immidiate_for_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x55, 0x69, 0xcc, 0x00]);
        assert_eq!(cpu.reg_a, 33);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x69_adc_immidiate_for_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x40, 0x69, 0x40, 0x00]);
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x29_and_with_immidiate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xd5, 0x29, 0xab, 0x00]);
        assert_eq!(cpu.reg_a, 0b1000_0001);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x0a_asl_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1110_0101, 0x0a, 0x00]);
        assert_eq!(cpu.reg_a, 0b1100_1010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x06_asl_zeropage() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1010_0101, 0x85, 0x03, 0x06, 0x03, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x03), 0b0100_1010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    // #[test]
    // fn test_0x90_bcc() {
    // }

    // #[test]
    // fn test_0xb0_bcs() {
    // }

    // #[test]
    // fn test_0xf0_beq() {
    // }

    #[test]
    fn test_0x24_bit_zeropage_for_v_not_nz() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x45, 0x85, 0x03, 0xa9, 0x01, 0x24, 0x03, 0x00]);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::OVERFLOW));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x24_bit_zeropage_for_nz_not_v() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x85, 0x03, 0xa9, 0x7f, 0x24, 0x03, 0x00]);
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    // #[test]
    // fn test_0x30_bmi() {
    // }

    // #[test]
    // fn test_0xd0_bne() {
    // }

    // #[test]
    // fn test_0x10_bpl() {
    // }

    // #[test]
    // fn test_0x50_bvc() {
    // }

    // #[test]
    // fn test_0x70_bvs() {
    // }

    #[test]
    fn test_0x18_clc() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x0a, 0x18, 0x00]);
        assert!(!cpu.status.contains(CpuFlags::CARRY))
    }

    // #[test]
    // fn test_0xd8_cld() {
    // }

    // #[test]
    // fn test_0x58_cli() {
    // }

    #[test]
    fn test_0xb8_clv() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x40, 0x69, 0x40, 0xb8, 0x00]);
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0xc9_cmp_immidiate_for_cn_not_z() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x88, 0xc9, 0x04, 0x00]);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xc9_cmp_immidiate_for_cz_not_n() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x08, 0xc9, 0x08, 0x00]);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xc9_cmp_immidiate_for_n_not_cz() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0xc9, 0x01, 0x00]);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    // #[test]
    // fn test_0x0e_cpx() {
    // }

    // #[test]
    // fn test_0xc0_cpy() {
    // }

    #[test]
    fn test_0xc6_dec_zeropage_for_not_nz() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x04, 0x85, 0x05, 0xc6, 0x05, 0x00]);
        let data = cpu.memory_read_u8(0x05);
        assert_eq!(data, 0x03);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xca_dex_for_z_not_n() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0xaa, 0xca, 0x00]);
        assert_eq!(cpu.reg_x, 0x00);
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x88_dey_for_n_not_z() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0xa8, 0x88, 0x00]);
        assert_eq!(cpu.reg_y, 0xff);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x49_eor_immidiate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x50, 0x49, 0x14, 0x00]);
        assert_eq!(cpu.reg_a, 0x44);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xe6_inc_zeropage_for_not_nz() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x04, 0x85, 0x05, 0xe6, 0x05, 0x00]);
        let data = cpu.memory_read_u8(0x05);
        assert_eq!(data, 0x05);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xe8_inx_for_z_not_n() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 0x00);
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xe8_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 1);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xc8_iny_for_n_not_z() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x7f, 0xa8, 0xc8, 0x00]);
        assert_eq!(cpu.reg_y, 0x80);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x4c_jmp_absolute() {
        let mut cpu = CPU::new();
        cpu.memory_write_u8(0x0000, 0xa9);
        cpu.memory_write_u8(0x0001, 0xaa);
        cpu.memory_write_u8(0x0002, 0x00);
        cpu.load_and_run(vec![0xa9, 0x55, 0x4c, 0x00, 0x00, 0x00]);
        assert_eq!(cpu.reg_a, 0xaa);
    }

    // #[test]
    // fn test_0x6c_jmp_indirect() {
    // }

    // #[test]
    // fn test_0x20_jsr() {
    // }

    #[test]
    fn test_0xa9_lda_immidiate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.status.contains(CpuFlags::ZERO) == false);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE) == false);
    }

    #[test]
    fn test_0xa9_lda_immidiate_for_z() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.contains(CpuFlags::ZERO) == true);
    }

    #[test]
    fn test_0xa5_lda_zeropage() {
        let mut cpu = CPU::new();
        cpu.memory_write_u8(0x10, 0x55);
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x55);
    }

    #[test]
    fn test_0xa2_ldx_immidiate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0xa5, 0x00]);
        assert_eq!(cpu.reg_x, 0xa5);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa0_ldy_immidiate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x5a, 0x00]);
        assert_eq!(cpu.reg_y, 0x5a);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x4a_lsr_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1110_0101, 0x4a, 0x00]);
        assert_eq!(cpu.reg_a, 0b0111_0010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x46_lsr_zeropage() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1010_0101, 0x85, 0x03, 0x46, 0x03, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x03), 0b0101_0010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0xea_nop() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xea, 0xa9, 0x55, 0x00]);
        assert_eq!(cpu.reg_a, 0x55);
    }

    #[test]
    fn test_0x09_ora_immmidiate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b0101_1010, 0x09, 0b1001_0100, 0x00]);
        assert_eq!(cpu.reg_a, 0b1101_1110);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x48_0x68_pha_pla() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x48, 0xa9, 0x00, 0x68, 0x00]);
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x08_0x28_php_plp() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x08, 0xa9, 0x00, 0x28, 0x00]);
        assert_eq!(cpu.reg_a, 0x00);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x2a_rol_accumulator_with_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xc9, 0x00, 0xa9, 0b1010_0000, 0x2a, 0x00]);
        assert_eq!(cpu.reg_a, 0b0100_0001);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x26_rol_zeropage_without_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b0010_0101, 0x85, 0x03, 0x26, 0x03, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x03), 0b0100_1010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x6a_ror_accumulator_with_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xc9, 0x00, 0xa9, 0b1010_0000, 0x6a, 0x00]);
        assert_eq!(cpu.reg_a, 0b1101_0000);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x66_ror_zeropage_without_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0b1010_0001, 0x85, 0x03, 0x66, 0x03, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x03), 0b0101_0000);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
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

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
        assert_eq!(cpu.reg_x, 10);
    }

    #[test]
    fn test_0xa8_tay_move_a_to_y() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0a, 0xa8, 0x00]);
        assert_eq!(cpu.reg_y, 10);
    }

    #[test]
    fn test_0xe8_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 0xc1);
    }
}

