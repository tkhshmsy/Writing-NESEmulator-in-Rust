use bitflags::*;
use std::collections::HashMap;
use crate::opcodes;
use crate::bus::Memory;
use crate::bus::Bus;

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
    pub bus: Bus
}

impl Memory for CPU {
    fn memory_read_u8(&self, addr: u16) -> u8 {
        return self.bus.memory_read_u8(addr);
    }

    fn memory_write_u8(&mut self, addr: u16, data: u8) {
        self.bus.memory_write_u8(addr, data);
    }
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        CPU {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_sp: STACK_RESET,
            status: CpuFlags::INTERRUPT_DISABLE | CpuFlags::BREAK2,
            reg_pc: 0,
            bus: bus,
        }
    }

    pub fn get_absolute_address(&self, mode: &AddressingMode, addr: u16) -> u16 {
        match mode {
            AddressingMode::Immediate => addr,
            AddressingMode::ZeroPage => self.bus.memory_read_u8(addr) as u16,
            AddressingMode::Absolute => self.bus.memory_read_u16(addr),

            AddressingMode::ZeroPage_X => {
                let pos = self.bus.memory_read_u8(addr);
                let addr = pos.wrapping_add(self.reg_x) as u16;
                return addr;
            },
            AddressingMode::ZeroPage_Y  => {
                let pos = self.bus.memory_read_u8(addr);
                let addr = pos.wrapping_add(self.reg_y) as u16;
                return addr;
            },
            AddressingMode::Absolute_X => {
                let base = self.bus.memory_read_u16(addr);
                let addr = base.wrapping_add(self.reg_x as u16);
                return addr;
            },
            AddressingMode::Absolute_Y => {
                let base = self.bus.memory_read_u16(addr);
                let addr = base.wrapping_add(self.reg_y as u16);
                return addr;
            },
            AddressingMode::Indirect_X => {
                let base = self.bus.memory_read_u8(addr);
                let ptr = (base as u8).wrapping_add(self.reg_x);
                let lo = self.bus.memory_read_u8(ptr as u16);
                let hi = self.bus.memory_read_u8(ptr.wrapping_add(1) as u16);
                return (hi as u16) << 8 | (lo as u16)
            },
            AddressingMode::Indirect_Y => {
                let base = self.bus.memory_read_u8(addr);
                let lo = self.bus.memory_read_u8(base as u16);
                let hi = self.bus.memory_read_u8((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.reg_y as u16);
                return deref;
            },
            _ => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.reg_pc,
            _ => self.get_absolute_address(mode, self.reg_pc),
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.memory_read_u8(addr);
        self.add_accumulator(value);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.memory_read_u8(addr);
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
        let mut value = self.bus.memory_read_u8(addr);
        self.status.set(CpuFlags::CARRY, value & 0x80 == 0x80);
        value = value << 1;
        self.bus.memory_write_u8(addr, value);
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
        let value = self.bus.memory_read_u8(addr);
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
        let value =  self.bus.memory_read_u8(addr);
        self.compare(self.reg_a, value);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let value =  self.bus.memory_read_u8(addr);
        self.compare(self.reg_x, value);
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let value =  self.bus.memory_read_u8(addr);
        self.compare(self.reg_y, value);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let mut value =  self.bus.memory_read_u8(addr);
        value = value.wrapping_sub(1);
        self.bus.memory_write_u8(addr, value);
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
        let value =  self.bus.memory_read_u8(addr);
        self.reg_a = self.reg_a ^ value;
        self.update_cpuflags(self.reg_a);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let mut value =  self.bus.memory_read_u8(addr);
        value = value.wrapping_add(1);
        self.bus.memory_write_u8(addr, value);
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
        let addr =  self.bus.memory_read_u16(self.reg_pc);
        self.reg_pc = addr;
    }

    fn jmp_indirect(&mut self) {
        let addr =  self.bus.memory_read_u16(self.reg_pc);
        let indirect = if addr & 0x00ff == 0x00ff {
            // reproduction 6502 bug
            let lo = self.bus.memory_read_u8(addr);
            let hi = self.bus.memory_read_u8(addr & 0xff00);
            (hi as u16) << 8  | (lo as u16)
        } else {
            self.bus.memory_read_u16(addr)
        };
        self.reg_pc = indirect;
    }

    fn jsr(&mut self) {
        self.stack_push_u16(self.reg_pc + 2 - 1);
        let addr = self.get_operand_address(&AddressingMode::Immediate);
        self.reg_pc = self.bus.memory_read_u16(addr);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.memory_read_u8(addr);
        self.reg_a = value;
        self.update_cpuflags(self.reg_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.memory_read_u8(addr);
        self.reg_x = value;
        self.update_cpuflags(self.reg_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.bus.memory_read_u8(addr);
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
        let mut value = self.bus.memory_read_u8(addr);
        self.status.set(CpuFlags::CARRY, value & 0x01 == 0x01);
        value = value >> 1;
        self.bus.memory_write_u8(addr, value);
        self.update_cpuflags(value);
    }

    fn nop(&mut self) {
        return;
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr =  self.get_operand_address(mode);
        let value =  self.bus.memory_read_u8(addr);
        self.reg_a = self.reg_a | value;
        self.update_cpuflags(self.reg_a);
    }

    fn pha(&mut self) {
        self.stack_push_u8(self.reg_a);
    }

    fn php(&mut self) {
        let mut flags = self.status.clone();
        flags.set(CpuFlags::BREAK1, true);
        flags.set(CpuFlags::BREAK2, true);
        self.stack_push_u8(flags.bits());
    }

    fn pla(&mut self) {
        self.reg_a = self.stack_pop_u8();
        self.update_cpuflags(self.reg_a);
    }

    fn plp(&mut self) {
        self.status.bits = self.stack_pop_u8();
        self.status.set(CpuFlags::BREAK1, false);
        self.status.set(CpuFlags::BREAK2, true);
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
        let mut value = self.bus.memory_read_u8(addr);
        let old_carry = self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY, value & 0x80== 0x80);
        value = value << 1;
        if old_carry {
            value |= 0x01;
        }
        self.bus.memory_write_u8(addr, value);
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
        let mut value = self.bus.memory_read_u8(addr);
        let old_carry = self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY, value & 0x01 == 0x01);
        value = value >> 1;
        if old_carry {
            value |= 0x80;
        }
        self.bus.memory_write_u8(addr, value);
        self.update_cpuflags(value);
    }

    fn rti(&mut self) {
        self.status.bits = self.stack_pop_u8();
        self.status.set(CpuFlags::BREAK1, false);
        self.status.set(CpuFlags::BREAK2, true);
        self.reg_pc = self.stack_pop_u16();
    }

    fn rts(&mut self) {
        self.reg_pc = self.stack_pop_u16() + 1;
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value0 = self.bus.memory_read_u8(addr);
        // Complement representation and subtract 1
        let value = (value0 as i8).wrapping_neg().wrapping_sub(1);
        self.add_accumulator(value as u8);
    }

    fn sec(&mut self) {
        self.status.set(CpuFlags::CARRY, true);
    }

    fn sed(&mut self) {
        self.status.set(CpuFlags::DECIMAL_MODE, true);
    }

    fn sei(&mut self) {
        self.status.set(CpuFlags::INTERRUPT_DISABLE, true);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.bus.memory_write_u8(addr, self.reg_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.bus.memory_write_u8(addr, self.reg_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.bus.memory_write_u8(addr, self.reg_y);
    }

    fn tax(&mut self) {
        self.reg_x = self.reg_a;
        self.update_cpuflags(self.reg_x);
    }

    fn tay(&mut self) {
        self.reg_y = self.reg_a;
        self.update_cpuflags(self.reg_y);
    }

    fn tsx(&mut self) {
        self.reg_x = self.reg_sp;
        self.update_cpuflags(self.reg_x);
    }

    fn txa(&mut self) {
        self.reg_a = self.reg_x;
        self.update_cpuflags(self.reg_a);
    }

    fn txs(&mut self) {
        self.reg_sp = self.reg_x;
    }

    fn tya(&mut self) {
        self.reg_a = self.reg_y;
        self.update_cpuflags(self.reg_a);
    }

    fn alr_unofficial(&mut self, mode: &AddressingMode) {
    }

    fn anc_unofficial(&mut self, mode: &AddressingMode) {
    }

    fn arr_unofficial(&mut self, mode: &AddressingMode) {
    }

    fn axs_unofficial(&mut self, mode: &AddressingMode) {
    }

    fn lax_unofficial(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.tax();
    }

    fn sax_unofficial(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.reg_a & self.reg_x;
        self.bus.memory_write_u8(addr, data);
        // self.update_cpuflags(data);
    }

    fn dcp_unofficial(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.bus.memory_read_u8(addr);
        data = data.wrapping_sub(1);
        self.bus.memory_write_u8(addr, data);
        self.status.set(CpuFlags::CARRY, data <= self.reg_a);
        self.update_cpuflags(self.reg_a.wrapping_sub(data));
    }

    fn isb_unofficial(&mut self, mode: &AddressingMode) {
        self.inc(mode);
        self.sbc(mode);
    }

    fn rla_unofficial(&mut self, mode: &AddressingMode) {
        self.rol(mode);
        self.and(mode);
    }

    fn rra_unofficial(&mut self, mode: &AddressingMode) {
    }

    fn slo_unofficial(&mut self, mode: &AddressingMode) {
        self.asl(mode);
        self.ora(mode);
    }

    fn sre_unofficial(&mut self, mode: &AddressingMode) {
        self.lsr(mode);
        self.eor(mode);
    }

    fn sbc_unofficial(&mut self, mode: &AddressingMode) {
        self.sbc(mode);
    }

    fn nop_unofficial(&mut self) {
        return;
    }

    fn nop_with_read_unofficial(&mut self) {
        // let addr = self.get_operand_address(mode);
        // let data = self.bus.memory_read_u8(addr);
        return;
    }

    fn update_cpuflags(&mut self, data: u8) {
        self.status.set(CpuFlags::ZERO, data == 0);
        self.status.set(CpuFlags::NEGATIVE, data & 0b1000_0000 != 0);
    }

    fn add_accumulator(&mut self, value: u8) {
        let sum = self.reg_a as u16 + value as u16 + if self.status.contains(CpuFlags::CARRY) { 1 as u16 } else { 0 as u16};
        self.status.set(CpuFlags::CARRY, sum > 0xff);
        let result = (sum & 0x00ff) as u8;
        self.status.set(CpuFlags::OVERFLOW, (result ^ value) & (result ^ self.reg_a) & 0x80 != 0x00 );
        self.reg_a = result;
        self.update_cpuflags(self.reg_a);
    }

    fn branch(&mut self) {
        let dst = self.bus.memory_read_u8(self.reg_pc) as i8;
        let addr = self.reg_pc.wrapping_add(1).wrapping_add(dst as u16);
        self.reg_pc = addr;
    }

    fn compare(&mut self, lhs: u8, rhs: u8) {
        self.status.set(CpuFlags::CARRY, lhs >= rhs);
        self.update_cpuflags(lhs.wrapping_sub(rhs));
    }

    fn stack_push_u8(&mut self, data: u8) {
        self.bus.memory_write_u8((STACK_BASE as u16) + (self.reg_sp as u16), data);
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
        return self.bus.memory_read_u8((STACK_BASE as u16) + (self.reg_sp as u16));
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
        self.status = CpuFlags::INTERRUPT_DISABLE | CpuFlags::BREAK2;
        self.reg_pc = self.bus.memory_read_u16(0xFFFC);
    }

    #[allow(dead_code)]
    fn load(&mut self, program: Vec<u8>) {
        // only for test code -> 0x0600
        for i in 0..(program.len() as u16) {
            self.bus.memory_write_u8(0x0600 + i, program[i as usize]);
        }
    }

    #[allow(dead_code)]
    fn load_and_run(&mut self, program: Vec<u8>) {
        // only for test
        self.load(program);
        self.reset();
        self.reg_pc = 0x0600;
        self.run();
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODE_MAP;

        loop {
            callback(self);

            let code = self.bus.memory_read_u8(self.reg_pc);
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
                0x20 => {
                    // JSR
                    self.jsr();
                },
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
                0x40 => {
                    // RTI
                    self.rti();
                },
                0x60 => {
                    // RTS
                    self.rts();
                },
                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => {
                    // SBC
                    self.sbc(&opcode.mode);
                },
                0x38 => {
                    // SEC
                    self.sec();
                },
                0xf8 => {
                    // SED
                    self.sed();
                },
                0x78 => {
                    // SEI
                    self.sei();
                },
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    // STA
                    self.sta(&opcode.mode);
                },
                0x86 | 0x96 | 0x8e => {
                    // STX
                    self.stx(&opcode.mode);
                },
                0x84 | 0x94 | 0x8c => {
                    //STY
                    self.sty(&opcode.mode);
                },
                0xaa => {
                    // TAX
                    self.tax();
                },
                0xa8 => {
                    // TAY
                    self.tay();
                },
                0xba => {
                    // TSX
                    self.tsx();
                },
                0x8a => {
                    // TXA
                    self.txa();
                },
                0x9a => {
                    // TXS
                    self.txs();
                },
                0x98 => {
                    // TYA
                    self.tya();
                },
                // ========== unofficial opcodes ==========
                0x4b => {
                    // ALR
                    self.alr_unofficial(&opcode.mode);
                },
                0x0b | 0x2b => {
                    // ANC
                    self.anc_unofficial(&opcode.mode);
                },
                0x6b => {
                    // ARR
                    self.arr_unofficial(&opcode.mode);
                },
                0xcb => {
                    // AXS
                    self.axs_unofficial(&opcode.mode);
                },
                0xa7 | 0xb7 | 0xaf | 0xbf | 0xa3 | 0xb3 => {
                    // LAX
                    self.lax_unofficial(&opcode.mode);
                },
                0x87 | 0x97 | 0x8f | 0x83 => {
                    // SAX
                    self.sax_unofficial(&opcode.mode);
                },
                0xc7 | 0xd7 | 0xcf | 0xdf | 0xdb | 0xd3 | 0xc3 => {
                    // DCP
                    self.dcp_unofficial(&opcode.mode);
                },
                0xe7 | 0xf7 | 0xef | 0xff | 0xfb | 0xe3 | 0xf3 => {
                    // ISB
                    self.isb_unofficial(&opcode.mode);
                },
                0x27 | 0x37 | 0x2f | 0x3f | 0x3b | 0x33 | 0x23 => {
                    // RLA
                    self.rla_unofficial(&opcode.mode);
                },
                0x67 | 0x77 | 0x6f | 0x7f | 0x7b | 0x63 | 0x73 => {
                    // RRA
                    self.rra_unofficial(&opcode.mode);
                },
                0x07 | 0x17 | 0x0f | 0x1f | 0x1b | 0x03 | 0x13 => {
                    // SLO
                    self.slo_unofficial(&opcode.mode);
                },
                0x47 | 0x57 | 0x4f | 0x5f | 0x5b | 0x43 | 0x53 => {
                    // SRE
                    self.sre_unofficial(&opcode.mode);
                },
                0xeb => {
                    // SBC
                    self.sbc_unofficial(&opcode.mode);
                },
                0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 => {
                    // NOP immediate
                    self.nop_unofficial();
                },
                0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4
                | 0x0c | 0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => {
                    // NOP with read
                    self.nop_with_read_unofficial();
                },
                | 0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72
                | 0x92 | 0xb2 | 0xd2 | 0xf2 | 0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => {
                    // NOP others
                    self.nop_unofficial();
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
    fn test_0x69_adc_immidiate_for_not_c() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x55, 0x69, 0x10, 0x00]);
        // 0x55 + 0x10 = 0x65, no CARRY, no OVERFLOW
        assert_eq!(cpu.reg_a, 0x65);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x69_adc_immidiate_for_c() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x55, 0x69, 0xcc, 0x00]);
        // 0x55 + 0xcc = 33+256, with CARRY, no OVERFLOW
        assert_eq!(cpu.reg_a, 33);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x69_adc_immidiate_for_v() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x40, 0x69, 0x40, 0x00]);
        // 64 + 64 = 128(=-128), with OVERFLOW
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x69_adc_immidiate_with_carry_for_cz() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xff, 0xc9, 0x00, 0xa9, 0xfe, 0x69, 0x01, 0x00]);
        // 0xfe + 0x01 + CARRY = 0x00 + 256 with CARRY, no OVERFLOW
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x69_adc_immidiate_with_carried_overflow() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xff, 0xc9, 0x00, 0xa9, 0x80, 0x69, 0xff, 0x00]);
        // 0x80 + 0xff + CARRY = 0x80 + 256 with CARRY, no OVERFLOW
        assert_eq!(cpu.reg_a, 0x80);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x29_and_with_immidiate() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xd5, 0x29, 0xab, 0x00]);
        assert_eq!(cpu.reg_a, 0b1000_0001);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x0a_asl_accumulator() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0b1110_0101, 0x0a, 0x00]);
        assert_eq!(cpu.reg_a, 0b1100_1010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x06_asl_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
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
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x45, 0x85, 0x03, 0xa9, 0x01, 0x24, 0x03, 0x00]);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::OVERFLOW));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x24_bit_zeropage_for_nz_not_v() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
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
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
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
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x40, 0x69, 0x40, 0xb8, 0x00]);
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0xc9_cmp_immidiate_for_cn_not_z() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x88, 0xc9, 0x04, 0x00]);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xc9_cmp_immidiate_for_cz_not_n() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x08, 0xc9, 0x08, 0x00]);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xc9_cmp_immidiate_for_n_not_cz() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x00, 0xc9, 0x01, 0x00]);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xe0_cpx_immidiate() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa2, 0x88, 0xe0, 0x04, 0x00]);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xc0_cpy_immidiate() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa0, 0x04, 0xc0, 0x88, 0x00]);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xc6_dec_zeropage_for_not_nz() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x04, 0x85, 0x05, 0xc6, 0x05, 0x00]);
        let data = cpu.memory_read_u8(0x05);
        assert_eq!(data, 0x03);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xca_dex_for_z_not_n() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x01, 0xaa, 0xca, 0x00]);
        assert_eq!(cpu.reg_x, 0x00);
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x88_dey_for_n_not_z() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x00, 0xa8, 0x88, 0x00]);
        assert_eq!(cpu.reg_y, 0xff);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x49_eor_immidiate() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x50, 0x49, 0x14, 0x00]);
        assert_eq!(cpu.reg_a, 0x44);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xe6_inc_zeropage_for_not_nz() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x04, 0x85, 0x05, 0xe6, 0x05, 0x00]);
        let data = cpu.memory_read_u8(0x05);
        assert_eq!(data, 0x05);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xe8_inx_for_z_not_n() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 0x00);
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xe8_inx_overflow() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 1);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xc8_iny_for_n_not_z() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x7f, 0xa8, 0xc8, 0x00]);
        assert_eq!(cpu.reg_y, 0x80);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x4c_jmp_absolute() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x0000, 0xa9);
        cpu.memory_write_u8(0x0001, 0xaa);
        cpu.memory_write_u8(0x0002, 0x00);
        cpu.load_and_run(vec![0xa9, 0x55, 0x4c, 0x00, 0x00, 0x00]);
        assert_eq!(cpu.reg_a, 0xaa);
    }

    // #[test]
    // fn test_0x6c_jmp_indirect() {
    // }

    #[test]
    fn test_0x20_0x60_jsr_rts() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x0010, 0xa9);
        cpu.memory_write_u8(0x0011, 0x02);
        cpu.memory_write_u8(0x0012, 0x60);
        cpu.load_and_run(vec![0xa9, 0x01, 0x20, 0x10, 0x00, 0xa2, 0x03, 0x00]);
        assert_eq!(cpu.reg_a, 0x02);
        assert_eq!(cpu.reg_x, 0x03);
    }

    #[test]
    fn test_0xa9_lda_immidiate() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.status.contains(CpuFlags::ZERO) == false);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE) == false);
    }

    #[test]
    fn test_0xa9_lda_immidiate_for_z() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.contains(CpuFlags::ZERO) == true);
    }

    #[test]
    fn test_0xa5_lda_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x10, 0x55);
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x55);
    }

    #[test]
    fn test_0xa2_ldx_immidiate() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa2, 0xa5, 0x00]);
        assert_eq!(cpu.reg_x, 0xa5);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa0_ldy_immidiate() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa0, 0x5a, 0x00]);
        assert_eq!(cpu.reg_y, 0x5a);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x4a_lsr_accumulator() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0b1110_0101, 0x4a, 0x00]);
        assert_eq!(cpu.reg_a, 0b0111_0010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x46_lsr_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0b1010_0101, 0x85, 0x03, 0x46, 0x03, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x03), 0b0101_0010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0xea_nop() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xea, 0xa9, 0x55, 0x00]);
        assert_eq!(cpu.reg_a, 0x55);
    }

    #[test]
    fn test_0x09_ora_immmidiate() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0b0101_1010, 0x09, 0b1001_0100, 0x00]);
        assert_eq!(cpu.reg_a, 0b1101_1110);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x48_0x68_pha_pla() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x80, 0x48, 0xa9, 0x00, 0x68, 0x00]);
        assert_eq!(cpu.reg_a, 0x80);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x08_0x28_php_plp() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x80, 0x08, 0xa9, 0x00, 0x28, 0x00]);
        assert_eq!(cpu.reg_a, 0x00);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x2a_rol_accumulator_with_carry() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xff, 0xc9, 0x00, 0xa9, 0b1010_0000, 0x2a, 0x00]);
        assert_eq!(cpu.reg_a, 0b0100_0001);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x26_rol_zeropage_without_carry() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0b0010_0101, 0x85, 0x03, 0x26, 0x03, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x03), 0b0100_1010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x6a_ror_accumulator_with_carry() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xff, 0xc9, 0x00, 0xa9, 0b1010_0000, 0x6a, 0x00]);
        assert_eq!(cpu.reg_a, 0b1101_0000);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x66_ror_zeropage_without_carry() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0b1010_0001, 0x85, 0x03, 0x66, 0x03, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x03), 0b0101_0000);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0xe9_sbc_immidiate_for_not_cz() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x10, 0xe9, 0x01, 0x00]);
        // 16 - 1 - 1 = 14, no CARRY
        assert_eq!(cpu.reg_a, 0x0e);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0xe9_sbc_immidiate_for_v_not_cz() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x80, 0xe9, 0x7f, 0x00]);
        // 0x80 - 0x7f - 1 = 0x00
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0xe9_sbc_immidiate_with_carry_for_zv_not_c() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x80, 0x38, 0xe9, 0x7f, 0x00]);
        // 0x80 - 0x7f = 0x01 with OVERFLOW, no CARRY
        assert_eq!(cpu.reg_a, 0x01);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0xe9_sbc_immidiate_with_carried_overflow() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x40, 0x38, 0xe9, 0xff, 0x00]);
        // 0x40 - 0xff = 0x41 with CARRY, no OVERFLOW
        assert_eq!(cpu.reg_a, 0x41);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0x38_sec() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0x38, 0x00]);
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0xf8_sed() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xf8, 0x00]);
        assert!(cpu.status.contains(CpuFlags::DECIMAL_MODE));
    }

    #[test]
    fn test_0x78_sed() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0x78, 0x00]);
        assert!(cpu.status.contains(CpuFlags::INTERRUPT_DISABLE));
    }

    #[test]
    fn test_0x85_sta_to_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x55, 0x85, 0x03, 0x00]);
        let data = cpu.memory_read_u8(0x03);
        assert_eq!(data, 0x55);
    }

    #[test]
    fn test_0x95_sta_to_zeropage_x() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x55, 0xaa, 0xa9, 0xaa, 0x95, 0x03, 0x00]);
        let data = cpu.memory_read_u8(0x58);
        assert_eq!(data, 0xaa);
    }

    #[test]
    fn test_0x86_stx_to_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa2, 0x55, 0x86, 0x03, 0x00]);
        let data = cpu.memory_read_u8(0x03);
        assert_eq!(data, 0x55);
    }

    #[test]
    fn test_0x84_sty_to_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa0, 0x55, 0x84, 0x03, 0x00]);
        let data = cpu.memory_read_u8(0x03);
        assert_eq!(data, 0x55);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
        assert_eq!(cpu.reg_x, 10);
    }

    #[test]
    fn test_0xa8_tay_move_a_to_y() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x0a, 0xa8, 0x00]);
        assert_eq!(cpu.reg_y, 10);
    }

    #[test]
    fn test_0xba_tsx() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0x48, 0xba, 0x00]);
        assert_eq!(cpu.reg_sp, cpu.reg_x);
    }

    #[test]
    fn test_0x8a_txa() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa2, 0x55, 0x8a, 0x00]);
        assert_eq!(cpu.reg_x, cpu.reg_a);
    }

    #[test]
    fn test_0x9a_txs() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0x48, 0xba, 0xe8, 0x9a, 0x00]);
        assert_eq!(cpu.reg_x, cpu.reg_sp);
    }

    #[test]
    fn test_0x98_tya() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa0, 0x55, 0x98, 0x00]);
        assert_eq!(cpu.reg_y, cpu.reg_a);
    }

    #[test]
    fn test_0xe8_5_ops_working_together() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 0xc1);
    }

    // ========== unofficial opcodes ==========
    #[test]
    fn test_0xa7_lax_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x10, 0x55);
        cpu.load_and_run(vec![0xa7, 0x10, 0x00]);
        assert_eq!(cpu.reg_a, 0x55);
        assert_eq!(cpu.reg_x, 0x55);
    }

    #[test]
    fn test_0x87_sax_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x55, 0xa2, 0xa5, 0x87, 0x10, 0x00]);
        assert_eq!(cpu.bus.memory_read_u8(0x10), 0x05);
        assert_eq!(cpu.reg_a, 0x55);
        assert_eq!(cpu.reg_x, 0xa5);
    }

    #[test]
    fn test_0xeb_sbc_immidiate_for_not_cz() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0x10, 0xeb, 0x01, 0x00]);
        // 16 - 1 - 1 = 14, no CARRY
        assert_eq!(cpu.reg_a, 0x0e);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::OVERFLOW))
    }

    #[test]
    fn test_0xc7_dcp_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x10, 0x55);
        cpu.load_and_run(vec![0xa9, 0x54, 0xc7, 0x10, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x10), 0x54);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0xe7_isb_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x10, 0x55);
        cpu.load_and_run(vec![0xa9, 0x57, 0xe7, 0x10, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x10), 0x56);
        assert_eq!(cpu.reg_a, 0x00);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_0x07_slo_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x10, 0b1010_0001);
        cpu.load_and_run(vec![0x38, 0xa9, 0b0100_1000, 0x07, 0x10, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x10), 0b0100_0010);
        assert_eq!(cpu.reg_a, 0b0100_1010);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x27_rla_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x10, 0b0010_0001);
        cpu.load_and_run(vec![0x38, 0xa9, 0b0100_1000, 0x27, 0x10, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x10), 0b0100_0011);
        assert_eq!(cpu.reg_a, 0b0100_0000);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x47_sre_zeropage() {
        let bus = Bus::new();
        let mut cpu = CPU::new(bus);
        cpu.memory_write_u8(0x10, 0b0010_0001);
        cpu.load_and_run(vec![0xa9, 0b0100_1000, 0x47, 0x10, 0x00]);
        assert_eq!(cpu.memory_read_u8(0x10), 0b0001_0000);
        assert_eq!(cpu.reg_a, 0b0101_1000);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }
}

