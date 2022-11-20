use bitflags::*;

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

pub struct CPU {
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_sp: u8,
    pub status: CpuFlags,
    pub reg_pc: u16,
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
        }
    }

    fn lda(&mut self, value: u8) {
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

    fn update_cpuflags(&mut self, result: u8) {
        self.status.set(CpuFlags::ZERO, if result == 0 { true } else { false });
        self.status.set(CpuFlags::NEGATIVE, if result & 0b1000_0000 != 0 { true } else { false });
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.reg_pc = 0;

        loop {
            let opcode = program[self.reg_pc as usize];
            self.reg_pc += 1;

            match opcode {
                0xA9 => {
                    let param = program[self.reg_pc as usize];
                    self.reg_pc += 1;
                    self.lda(param);
                }
                0xAA => {
                    self.tax();
                }
                0xE8 => {
                    self.inx();
                }
                0x00 => {
                    // BRK
                    return;
                }
                _ => todo!()
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
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0x05);
        assert!(cpu.status.contains(CpuFlags::ZERO) == false);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE) == false);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status.contains(CpuFlags::ZERO) == true);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.reg_a = 10;
        cpu.interpret(vec![0xaa, 0x00]);
        assert_eq!(cpu.reg_x, 10);
    }

    #[test]
    fn test_0xe8_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 0xc1);
    }

    #[test]
    fn test_0xe8_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.reg_x = 0xff;
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.reg_x, 1);
    }
}

