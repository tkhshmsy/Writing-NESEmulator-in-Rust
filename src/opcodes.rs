use crate::cpu::AddressingMode;
use std::collections::HashMap;

pub struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub len: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

impl OpCode {
    fn new(code: u8, mnemonic:&'static str, len: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode {
            code: code,
            mnemonic: mnemonic, 
            len: len,
            cycles: cycles,
            mode: mode,
        }
    }
}

lazy_static! {
    pub static ref CPU_OPCODES: Vec<OpCode> = vec![
        // ADC -  Add with Carry
        OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7D, "ADC", 3, 4, AddressingMode::Absolute_X), // cycle + 1 if page crossed
        OpCode::new(0x79, "ADC", 3, 4, AddressingMode::Absolute_Y), // cycle + 1 if page crossed
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x71, "ADC", 2, 5, AddressingMode::Indirect_Y), // cycle + 1 if page crossed

        // AND - Logical AND
        OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3D, "AND", 3, 4, AddressingMode::Absolute_X), // cycle + 1 if page crossed
        OpCode::new(0x39, "AND", 3, 4, AddressingMode::Absolute_Y), // cycle + 1 if page crossed
        OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x31, "AND", 2, 5, AddressingMode::Indirect_Y), // cycle + 1 if page crossed

        // ASL - Arithmetic Shift Left
        OpCode::new(0x0A, "ASL", 1, 2, AddressingMode::NonAddressing),
        OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1E, "ASL", 3, 7, AddressingMode::Absolute_X),

        // BCC - Branch if Carry Clear
        OpCode::new(0x90, "BCC", 2, 2, AddressingMode::NonAddressing), // cycle +1 if success, +2 if to new page

        // BCS - Branch if Carry Set
        OpCode::new(0xB0, "BCS", 2, 2, AddressingMode::NonAddressing), // cycle +1 if success, +2 if to new page

        // BEQ - Branch if Equal
        OpCode::new(0xF0, "BEQ", 2, 2, AddressingMode::NonAddressing), // cycle +1 if success, +2 if to new page

        // BIT - Bit Test
        OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute),

        // BMI - Branch if Minus
        OpCode::new(0x30, "BMI", 2, 2, AddressingMode::NonAddressing), // cycle +1 if success, +2 if to new page

        // BNE - Branch if Not Equal
        OpCode::new(0xD0, "BNE", 2, 2, AddressingMode::NonAddressing), // cycle +1 if success, +2 if to new page

        // BPL - Branch if Positive
        OpCode::new(0x10, "BPL", 2, 2, AddressingMode::NonAddressing), // cycle +1 if success, +2 if to new page

        // BRK - Force Interrupt
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NonAddressing),

        // BVC - Branch if Overflow Clear
        OpCode::new(0x50, "BVC", 2, 2, AddressingMode::NonAddressing), // cycle +1 if success, +2 if to new page

        // BVS - Branch if Overflow Set
        OpCode::new(0x70, "BVS", 2, 2, AddressingMode::NonAddressing), // cycle +1 if success, +2 if to new page

        // CLC - Clear Carry Flag
        OpCode::new(0x18, "CLC", 1, 2, AddressingMode::NonAddressing),

        // CLD - Clear Decimal Mode
        OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::NonAddressing),

        // CLI - Clear Interrupt Disable
        OpCode::new(0x58, "CLI", 1, 2, AddressingMode::NonAddressing),

        // CLV - Clear Overflow Flag
        OpCode::new(0xB8, "CLV", 1, 2, AddressingMode::NonAddressing),

        // CMP - Compare
        OpCode::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xDD, "CMP", 3, 4, AddressingMode::Absolute_X), // cycle + 1 if page crossed
        OpCode::new(0xD9, "CMP", 3, 4, AddressingMode::Absolute_Y), // cycle + 1 if page crossed
        OpCode::new(0xC1, "CMP", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xD1, "CMP", 2, 5, AddressingMode::Indirect_Y), // cycle + 1 if page crossed

        // CPX - Compare X Register
        OpCode::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute),

        // CPY - Compare Y Register
        OpCode::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xCC, "CPY", 3, 4, AddressingMode::Absolute),

        // DEC - Decrement Memory
        OpCode::new(0xC6, "DEC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xD6, "DEC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xCE, "DEC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xDE, "DEC", 3, 7, AddressingMode::Absolute_X),

        // DEX - Decrement X Register
        OpCode::new(0xCA, "DEX", 1, 2, AddressingMode::NonAddressing),

        // DEY - Decrement Y Register
        OpCode::new(0x88, "DEX", 1, 2, AddressingMode::NonAddressing),

        // EOR - Exclusive OR
        OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x4D, "EOR", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5D, "EOR", 3, 4, AddressingMode::Absolute_X), // cycle + 1 if page crossed
        OpCode::new(0x59, "EOR", 3, 4, AddressingMode::Absolute_Y), // cycle + 1 if page crossed
        OpCode::new(0x41, "EOR", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x51, "EOR", 2, 5, AddressingMode::Indirect_Y), // cycle + 1 if page crossed

        // INC - Increment Memory
        OpCode::new(0xE6, "INC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xF6, "INC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xEE, "INC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xFE, "INC", 3, 7, AddressingMode::Absolute_X),

        // INX - Increment X Register
        OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NonAddressing),

        // INY - Increment Y Register
        OpCode::new(0xC8, "INY", 1, 2, AddressingMode::NonAddressing),

        // JMP - Jump
        OpCode::new(0x4C, "JMP", 3, 3, AddressingMode::NonAddressing),
        OpCode::new(0x6C, "JMP", 3, 3, AddressingMode::NonAddressing),

        // JSR - Jump to Sub Routine
        OpCode::new(0x20, "JSR", 3, 6, AddressingMode::NonAddressing),

        // LDA - Load Accumulator
        OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::Absolute_X), // cycle + 1 if page crossed
        OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::Absolute_Y), // cycle + 1 if page crossed
        OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xB1, "LDA", 2, 5, AddressingMode::Indirect_Y), // cycle + 1 if page crossed

        // LDX - Load X Register
        OpCode::new(0xA2, "LDX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA6, "LDX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB6, "LDX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xAE, "LDX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBE, "LDX", 3, 4, AddressingMode::Absolute_Y), // cycle + 1 if page crossed

        // LDY - Load Y Register
        OpCode::new(0xA0, "LDY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA4, "LDY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB4, "LDY", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAC, "LDY", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBC, "LDY", 3, 4, AddressingMode::Absolute_X), // cycle + 1 if page crossed

        // LSR - Logical Shift Right
        OpCode::new(0x4A, "LSR", 1, 2, AddressingMode::NonAddressing),
        OpCode::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4E, "LSR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5E, "LSR", 3, 7, AddressingMode::Absolute_X),

        // NOP - No Operation
        OpCode::new(0xEA, "NOP", 1, 2, AddressingMode::NonAddressing),

        // ORA - Logical Inclusive OR
        OpCode::new(0x09, "ORA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0D, "ORA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1D, "ORA", 3, 4, AddressingMode::Absolute_X), // cycle + 1 if page crossed
        OpCode::new(0x19, "ORA", 3, 4, AddressingMode::Absolute_Y), // cycle + 1 if page crossed
        OpCode::new(0x01, "ORA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x11, "ORA", 2, 5, AddressingMode::Indirect_Y), // cycle + 1 if page crossed

        // PHA - Push Accumulator
        OpCode::new(0x48, "PHA", 1, 3, AddressingMode::NonAddressing),

        // PHP - Push Processor Status
        OpCode::new(0x08, "PHP", 1, 3, AddressingMode::NonAddressing),

        // PLA - Pull Accumulator
        OpCode::new(0x68, "PLA", 1, 4, AddressingMode::NonAddressing),

        // PLP - Pull Processor Status
        OpCode::new(0x28, "PHP", 1, 4, AddressingMode::NonAddressing),

        // ROL - Rotate Left
        OpCode::new(0x2A, "ROL", 1, 2, AddressingMode::NonAddressing),
        OpCode::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2E, "ROL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3E, "ROL", 3, 7, AddressingMode::Absolute_X),

        // ROR - Rotate Right
        OpCode::new(0x6A, "ROR", 1, 2, AddressingMode::NonAddressing),
        OpCode::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6E, "ROR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7E, "ROR", 3, 7, AddressingMode::Absolute_X),

        // RTI - Return from Interrupt
        OpCode::new(0x40, "RTI", 1, 6, AddressingMode::NonAddressing),

        // RTS - Return from Subroutine
        OpCode::new(0x60, "RTI", 1, 6, AddressingMode::NonAddressing),

        // SBC - Subtract with Carry
        OpCode::new(0xE9, "SBC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE5, "SBC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xF5, "SBC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xED, "SBC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xFD, "SBC", 3, 4, AddressingMode::Absolute_X), // cycle + 1 if page crossed
        OpCode::new(0xF9, "SBC", 3, 4, AddressingMode::Absolute_Y), // cycle + 1 if page crossed
        OpCode::new(0xE1, "SBC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xF1, "SBC", 2, 5, AddressingMode::Indirect_Y), // cycle + 1 if page crossed

        // SEC - Set Carry Flag
        OpCode::new(0x38, "SEC", 1, 2, AddressingMode::NonAddressing),

        // SED - Set Decimal Flag
        OpCode::new(0xF8, "SEC", 1, 2, AddressingMode::NonAddressing),

        // SEI - Set Interrupt Disable
        OpCode::new(0x78, "SEI", 1, 2, AddressingMode::NonAddressing),

        // STA - Store Accumulator
        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8d, "STA", 2, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, "STA", 2, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA", 2, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),

        // STX - Store X Register
        OpCode::new(0x86, "STX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x96, "STX", 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x8E, "STX", 3, 4, AddressingMode::Absolute),

        // STY - Store Y Register
        OpCode::new(0x84, "STY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x94, "STY", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8C, "STY", 3, 4, AddressingMode::Absolute),

        // TAX - Transfer Accumulator to X
        OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NonAddressing),

        // TAY - Transfer Accumulator to Y
        OpCode::new(0xA8, "TAY", 1, 2, AddressingMode::NonAddressing),

        // TSX - Transfer Stack Pointer to X
        OpCode::new(0xBA, "TSX", 1, 2, AddressingMode::NonAddressing),

        // TXA - Transfer X to Accumulator
        OpCode::new(0x8A, "TXA", 1, 2, AddressingMode::NonAddressing),

        // TXS - Transfer X to Stack Pointer
        OpCode::new(0x9A, "TXS", 1, 2, AddressingMode::NonAddressing),

        // TYA - Transfer Y to Accumulator
        OpCode::new(0x98, "TYA", 1, 2, AddressingMode::NonAddressing),
    ];

    pub static ref OPCODE_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for opcode in &*CPU_OPCODES {
            map.insert(opcode.code, opcode);
        }
        return map;
    };
}