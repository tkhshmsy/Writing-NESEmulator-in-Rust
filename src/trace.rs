use crate::cpu::CPU;
use crate::cpu::AddressingMode;
use crate::bus::Memory;
use crate::opcodes;
use crate::joypad::Joypad;
use std::collections::HashMap;

pub fn trace(cpu: &mut CPU) -> String {
    let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODE_MAP;
    let code = cpu.memory_read_u8(cpu.reg_pc);
    let ops = opcodes.get(&code).unwrap();

    let begin  = cpu.reg_pc;
    let mut dump = vec![];
    dump.push(code);

    let (operand_addr, value) = match ops.mode {
        AddressingMode::Immediate | AddressingMode::NonAddressing => (0, 0),
        _ => {
            let abs_addr = cpu.get_absolute_address(&ops.mode, begin + 1);
            (abs_addr, cpu.memory_read_u8(abs_addr))
        },
    };
    let operand_string = match ops.len {
        1 => match ops.code {
            0x0a | 0x4a | 0x2a | 0x6a => format!("A "),
            _ => String::from(""),
        },
        2 => {
            let addr = cpu.memory_read_u8(begin + 1);
            dump.push(addr);
            match ops.mode {
                AddressingMode::Immediate => format!("#${:02x}",addr),
                AddressingMode::ZeroPage => format!("${:02x} = {:02x}", operand_addr, value),
                AddressingMode::ZeroPage_X => format!("${:02x},X @ {:02x} = {:02x}", addr, operand_addr, value),
                AddressingMode::ZeroPage_Y => format!("${:02x},Y @ {:02x} = {:02x}", addr, operand_addr, value),
                AddressingMode::Indirect_X => format!("(${:02x},X) @ {:02x} = {:04x} = {:02x}", addr, addr.wrapping_add(cpu.reg_x), operand_addr, value),
                AddressingMode::Indirect_Y => format!("(${:02x}),Y = {:04x} @ {:04x} = {:02x}", addr, operand_addr.wrapping_sub(cpu.reg_y as u16), operand_addr, value),
                AddressingMode::NonAddressing => {
                    let tmp = (begin as usize + 2).wrapping_add((addr as i8) as usize);
                    format!("${:04x}", tmp)
                },
                _ => panic!("invalid addressing mode {:?} with length 2, code {:02x}", ops.mode, ops.code),
            }
        },
        3 => {
            let addr = cpu.memory_read_u16(begin + 1);
            dump.push((addr & 0x00FF) as u8);
            dump.push(((addr & 0xFF00) >> 8) as u8);
            match ops.mode {
                AddressingMode::NonAddressing => {
                    if ops.code == 0x6c {
                        let jmp_addr = if addr & 0x00FF == 0x00FF {
                            let lo = cpu.memory_read_u8(addr);
                            let hi = cpu.memory_read_u8(addr & 0xFF00);
                            ((hi as u16) << 8) | (lo as u16)
                        } else {
                            cpu.memory_read_u16(addr)
                        };
                        format!("(${:04x}) = {:04x}", addr, jmp_addr)
                    } else {
                        format!("${:04x}", addr)
                    }
                },
                AddressingMode::Absolute => format!("${:04x} = {:02x}", operand_addr, value),
                AddressingMode::Absolute_X => format!("${:04x},X @ {:04x} = {:02x}", addr, operand_addr, value),
                AddressingMode::Absolute_Y => format!("${:04x},Y @ {:04x} = {:02x}", addr, operand_addr, value),
                _ => panic!("invalid addressing mode {:?} with length 3, code {:02x}", ops.mode, ops.code),
            }
        },
        _ => String::from(""),
    };
    let hex_string = dump.iter().map(|z| format!("{:02x}", z)).collect::<Vec<String>>().join(" ");
    let asm_string = format!("{:04x}  {:8} {: >4} {}", begin, hex_string, ops.mnemonic, operand_string).trim().to_string();

    format!("{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
                asm_string, cpu.reg_a, cpu.reg_x, cpu.reg_y, cpu.status, cpu.reg_sp).to_ascii_uppercase()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bus::Bus;
    use crate::ppu::NesPPU;
    use crate::rom::test::test_rom;

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::new_with_rom(test_rom(), |_ppu: &NesPPU, _joypad: &mut Joypad|{});
        bus.memory_write_u8(100, 0xa2);
        bus.memory_write_u8(101, 0x01);
        bus.memory_write_u8(102, 0xca);
        bus.memory_write_u8(103, 0x88);
        bus.memory_write_u8(104, 0x00);

        let mut cpu = CPU::new(bus);
        cpu.reg_pc = 0x64;
        cpu.reg_a = 1;
        cpu.reg_x = 2;
        cpu.reg_y = 3;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
            result[0]
        );
        assert_eq!(
            "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
            result[1]
        );
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
            result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::new_with_rom(test_rom(), |_ppu: &NesPPU, _joypad: &mut Joypad|{});
        // ORA ($33), Y
        bus.memory_write_u8(100, 0x11);
        bus.memory_write_u8(101, 0x33);

        //data
        bus.memory_write_u8(0x33, 00);
        bus.memory_write_u8(0x34, 04);

        //target cell
        bus.memory_write_u8(0x400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.reg_pc = 0x64;
        cpu.reg_y = 0;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
            result[0]
        );
    }
}