#![allow(dead_code)]

/// 2^16 locations, each containing one word (16 bits).
/// Numbered from 0x0000 -> 0xFFFF
const MEMORY_ADDRESS_SPACE: [u16; 0xFFFF] = [0; 0xFFFF];

enum Opcodes {
    Add = 0b0001,
    And = 0b0101,
    Br = 0b0000,
    Jmp = 0b1100,

    JSR = 0b0100,
    LD = 0b0010,
    LDI = 0b1010,
    LDR = 0b0110,
    LEA = 0b1110,
    NOT = 0b1001,
    RTI = 0b1000,
    ST = 0b0011,
    STI = 0b1011,
    STR = 0b0111,
    TRAP = 0b1111,
    ILLEGAL = 0b1101,
}

#[derive(Debug, PartialEq)]
enum Instruction {
    Add {
        dest: u16,
        source_1: u16,
        source_2: u16,
    },
    AddImmediate {
        dest: u16,
        source: u16,
        value: u16,
    },
    And {
        dest: u16,
        source_1: u16,
        source_2: u16,
    },
    AndImmediate {
        dest: u16,
        source: u16,
        value: u16,
    },

    Br {
        n: bool,
        z: bool,
        p: bool,
        pc_offset: u16,
    },
    Jmp {
        base: u16,
    },
    Ret,

    JSR,
    LD,
    LDI,
    LDR,
    LEA,
    NOT,
    RET,
    RTI,
    ST,
    STI,
    STR,
    TRAP,
    ILLEGAL,
}

// indices are from 15 (leftmost) to 0 (rightmost):
// [15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00]
fn slice_bits(instruction: u16, from: u16, to: u16) -> u16 {
    let slice_size = from - to + 1;
    let mask = (1 << slice_size) - 1;
    (instruction >> to) & mask
}

fn set(instruction: u16, bit: u16) -> bool {
    instruction & (1 << bit) == (1 << bit)
}

/// expects a 5 bit number
fn sign_extend_5(x: u16) -> u16 {
    assert!(x < 0b100000, "must be a 5 bit number or smaller");
    if set(x, 4) {
        x | 0b1111111111100000
    } else {
        x
    }
}

/// expects a 9 bit number
fn sign_extend_9(x: u16) -> u16 {
    assert!(x < 0b1000000000, "must be a 9 bit number or smaller");
    if set(x as u16, 8) {
        (x as u16) | 0b1111111000000000
    } else {
        println!("{:#b}", x);
        x as u16
    }
}

impl Instruction {
    fn new(instruction: u16) -> Instruction {
        let opcode = slice_bits(instruction, 15, 12);
        let opcode: Opcodes = unsafe { std::mem::transmute(opcode as u8) };

        match opcode {
            Opcodes::Add => {
                if set(instruction, 5) {
                    Instruction::AddImmediate {
                        dest: slice_bits(instruction, 11, 9),
                        source: slice_bits(instruction, 8, 6),
                        value: sign_extend_5(slice_bits(instruction, 4, 0)),
                    }
                } else {
                    Instruction::Add {
                        dest: slice_bits(instruction, 11, 9),
                        source_1: slice_bits(instruction, 8, 6),
                        source_2: slice_bits(instruction, 2, 0),
                    }
                }
            }
            Opcodes::And => {
                if set(instruction, 5) {
                    Instruction::AndImmediate {
                        dest: slice_bits(instruction, 11, 9),
                        source: slice_bits(instruction, 8, 6),
                        value: sign_extend_5(slice_bits(instruction, 4, 0)),
                    }
                } else {
                    Instruction::And {
                        dest: slice_bits(instruction, 11, 9),
                        source_1: slice_bits(instruction, 8, 6),
                        source_2: slice_bits(instruction, 2, 0),
                    }
                }
            }
            Opcodes::Br => {
                Instruction::Br {
                    n: set(instruction, 11),
                    z: set(instruction, 10),
                    p: set(instruction, 9),
                    pc_offset: sign_extend_9(slice_bits(instruction, 8, 0)),
                }
            }
            Opcodes::Jmp => {
                let base = slice_bits(instruction, 8, 6);
                if base == 0b111 {
                    Instruction::Ret
                } else {
                    Instruction::Jmp { base }
                }
            }
            _ => Instruction::ILLEGAL,
        }
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_bits() {
        assert_eq!(slice_bits(0b1111_0000_0000_0000, 15, 12), 0b1111);
        assert_eq!(slice_bits(0b0000_1111_0000_0000, 11, 8), 0b1111);
        assert_eq!(slice_bits(0b0000_0000_1111_0000, 7, 4), 0b1111);
        assert_eq!(slice_bits(0b0000_0000_0000_1111, 3, 0), 0b1111);
    }

    #[test]
    fn test_set() {
        assert!(set(0b1, 0));
        assert!(set(0b10001, 4));
    }

    #[test]
    fn test_sign_extend_5() {
        assert_eq!(sign_extend_5(0b10001), 0b1111111111110001);
        assert_eq!(sign_extend_5(0b1001), 0b1001);
    }

    #[test]
    fn test_sign_extend_9() {
        assert_eq!(sign_extend_9(0b1_1000_0001), 0b111111111000_0001);
        assert_eq!(sign_extend_9(0b0_1000_0001), 0b1000_0001);
    }

    #[test]
    fn test_parse_instruction() {
        assert_eq!(
            Instruction::new(0b0001_100_010_0_00_001),
            Instruction::Add {
                dest: 0b100,
                source_1: 0b010,
                source_2: 0b001,
            }
        );

        assert_eq!(
            Instruction::new(0b0001_100_010_1_10001),
            Instruction::AddImmediate {
                dest: 0b100,
                source: 0b010,
                value: 0b1111111111110001,
            }
        );

        assert_eq!(
            Instruction::new(0b0101_100_010_0_00_001),
            Instruction::And {
                dest: 0b100,
                source_1: 0b010,
                source_2: 0b001,
            }
        );

        assert_eq!(
            Instruction::new(0b0101_100_010_1_01001),
            Instruction::AndImmediate {
                dest: 0b100,
                source: 0b10,
                value: 0b1001,
            }
        );

        assert_eq!(
            Instruction::new(0b0000_000_000000000),
            Instruction::Br {
                n: false,
                z: false,
                p: false,
                pc_offset: 0,
            }
        );

        assert_eq!(
            Instruction::new(0b0000_111_000000000),
            Instruction::Br {
                n: true,
                z: true,
                p: true,
                pc_offset: 0,
            }
        );

        assert_eq!(
            Instruction::new(0b0000_000_000001000),
            Instruction::Br {
                n: false,
                z: false,
                p: false,
                pc_offset: 0b1000,
            }
        );

        assert_eq!(
            Instruction::new(0b1100_000_010_000000),
            Instruction::Jmp {
                base: 0b010,
            }
        );

        assert_eq!(
            Instruction::new(0b1100_000_111_000000),
            Instruction::Ret,
        );
    }
}
