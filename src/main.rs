/// 2^16 locations, each containing one word (16 bits).
/// Numbered from 0x0000 -> 0xFFFF
const MEMORY_ADDRESS_SPACE: [u16; 0xFFFF] = [0; 0xFFFF];

enum Opcodes {
    Add = 0b0001,
    And = 0b0101,
    Br = 0b0000,
    Jmp = 0b1100,
    Jsr = 0b0100,
    Ld = 0b0010,
    LdI = 0b1010,
    LdR = 0b0110,
    Lea = 0b1110,
    Not = 0b1001,
    Rti = 0b1000,
    St = 0b0011,
    StI = 0b1011,
    Str = 0b0111,
    Trap = 0b1111,
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
    Jsr {
        pc_offset: u16,
    },
    JsrR {
        base: u16,
    },
    Ld {
        dest: u16,
        pc_offset: u16,
    },
    LdI {
        dest: u16,
        pc_offset: u16,
    },
    LdR {
        dest: u16,
        base: u16,
        offset: u16,
    },
    Lea {
        dest: u16,
        pc_offset: u16,
    },
    Not {
        dest: u16,
        source: u16,
    },
    Rti,
    St {
        source: u16,
        pc_offset: u16,
    },
    StI {
        source: u16,
        pc_offset: u16,
    },
    StR {
        source: u16,
        base: u16,
        offset: u16,
    },
    Trap {
        vec: u16,
    },
    Illegal,
}

// indices are from 15 (leftmost) to 0 (rightmost):
// [15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00]
fn slice_bits(instruction: u16, from: u16, to: u16) -> u16 {
    let slice_size = from - to + 1;
    let mask = (1 << slice_size) - 1;
    (instruction >> to) & mask
}

fn is_bit_set(instruction: u16, bit: u16) -> bool {
    instruction & (1 << bit) == (1 << bit)
}

fn sign_extend(n: u16, size: u16) -> u16 {
   if is_bit_set(n, size - 1) {
        n | (0b1111_1111_1111_1111 ^ ((1 << size) - 1))
    } else {
        n
    }
}

impl Instruction {
    fn new(instruction: u16) -> Instruction {
        let opcode = slice_bits(instruction, 15, 12);
        let opcode: Opcodes = unsafe { std::mem::transmute(opcode as u8) };

        match opcode {
            Opcodes::Add => {
                if is_bit_set(instruction, 5) {
                    Instruction::AddImmediate {
                        dest: slice_bits(instruction, 11, 9),
                        source: slice_bits(instruction, 8, 6),
                        value: sign_extend(slice_bits(instruction, 4, 0), 5),
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
                if is_bit_set(instruction, 5) {
                    Instruction::AndImmediate {
                        dest: slice_bits(instruction, 11, 9),
                        source: slice_bits(instruction, 8, 6),
                        value: sign_extend(slice_bits(instruction, 4, 0), 5),
                    }
                } else {
                    Instruction::And {
                        dest: slice_bits(instruction, 11, 9),
                        source_1: slice_bits(instruction, 8, 6),
                        source_2: slice_bits(instruction, 2, 0),
                    }
                }
            }
            Opcodes::Br => Instruction::Br {
                n: is_bit_set(instruction, 11),
                z: is_bit_set(instruction, 10),
                p: is_bit_set(instruction, 9),
                pc_offset: sign_extend(slice_bits(instruction, 8, 0), 9),
            },
            Opcodes::Jmp => {
                let base = slice_bits(instruction, 8, 6);
                if base == 0b111 {
                    Instruction::Ret
                } else {
                    Instruction::Jmp { base }
                }
            }
            Opcodes::Jsr => {
                if is_bit_set(instruction, 11) {
                    Instruction::Jsr {
                        pc_offset: sign_extend(slice_bits(instruction, 10, 0), 11),
                    }
                } else {
                    Instruction::JsrR {
                        base: slice_bits(instruction, 8, 6),
                    }
                }
            }
            Opcodes::Ld => Instruction::Ld {
                dest: slice_bits(instruction, 11, 9),
                pc_offset: sign_extend(slice_bits(instruction, 8, 0), 9),
            },
            Opcodes::LdI => Instruction::LdI {
                dest: slice_bits(instruction, 11, 9),
                pc_offset: sign_extend(slice_bits(instruction, 8, 0), 9),
            },
            Opcodes::LdR => Instruction::LdR {
                dest: slice_bits(instruction, 11, 9),
                base: slice_bits(instruction, 8, 6),
                offset: sign_extend(slice_bits(instruction, 5, 0), 6),
            },
            Opcodes::Lea => Instruction::Lea {
                dest: slice_bits(instruction, 11, 9),
                pc_offset: sign_extend(slice_bits(instruction, 8, 0), 9),
            },
            Opcodes::Not => Instruction::Not {
                dest: slice_bits(instruction, 11, 9),
                source: slice_bits(instruction, 8, 6),
            },
            Opcodes::Rti => Instruction::Rti,
            Opcodes::St => Instruction::St {
                source: slice_bits(instruction, 11, 9),
                pc_offset: sign_extend(slice_bits(instruction, 8, 0), 9),
            },
            Opcodes::StI => Instruction::StI {
                source: slice_bits(instruction, 11, 9),
                pc_offset: sign_extend(slice_bits(instruction, 8, 0), 9),
            },
            Opcodes::Str => Instruction::StR {
                source: slice_bits(instruction, 11, 9),
                base: slice_bits(instruction, 8, 6),
                offset: sign_extend(slice_bits(instruction, 5, 0), 6),
            },
            Opcodes::Trap => Instruction::Trap {
                vec: slice_bits(instruction, 7, 0),
            },
            _ => Instruction::Illegal,
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
        assert!(is_bit_set(0b1, 0));
        assert!(is_bit_set(0b10001, 4));
    }

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b10001, 5), 0b1111111111110001);
        assert_eq!(sign_extend(0b1001, 5), 0b1001);

        assert_eq!(sign_extend(0b1_1000_0001, 9), 0b111111111000_0001);
        assert_eq!(sign_extend(0b0_1000_0001, 9), 0b1000_0001);
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
            Instruction::Jmp { base: 0b010 }
        );

        assert_eq!(Instruction::new(0b1100_000_111_000000), Instruction::Ret,);

        assert_eq!(
            Instruction::new(0b0100_1_01000000001),
            Instruction::Jsr { pc_offset: 0b1000000001 },
        );

        assert_eq!(
            Instruction::new(0b0100_0_00_010_000000),
            Instruction::JsrR { base: 0b010 },
        );

        assert_eq!(
            Instruction::new(0b0010_010_010000001),
            Instruction::Ld { dest: 0b010, pc_offset: 0b10000001 },
        );

        assert_eq!(
            Instruction::new(0b1010_010_010000001),
            Instruction::LdI { dest: 0b010, pc_offset: 0b10000001 },
        );

        assert_eq!(
            Instruction::new(0b0110_010_010_100000),
            Instruction::LdR {
                dest: 0b010,
                base: 0b010,
                offset: 0b1111_1111_1110_0000,
            },
        );

        assert_eq!(
            Instruction::new(0b1110_010_010100000),
            Instruction::Lea {
                dest: 0b010,
                pc_offset: 0b10100000,
            },
        );

        assert_eq!(
            Instruction::new(0b1001_010_010_000000),
            Instruction::Not {
                dest: 0b010,
                source: 0b010,
            },
        );

        assert_eq!(
            Instruction::new(0b1000_000000000000),
            Instruction::Rti,
        );

        assert_eq!(
            Instruction::new(0b0011_010_100000000),
            Instruction::St {
                source: 0b010,
                pc_offset: 0b1111_1111_0000_0000,
            },
        );

        assert_eq!(
            Instruction::new(0b1011_010_100000000),
            Instruction::StI {
                source: 0b010,
                pc_offset: 0b1111_1111_0000_0000,
            },
        );

        assert_eq!(
            Instruction::new(0b0111_010_010_100000),
            Instruction::StR {
                source: 0b010,
                base: 0b010,
                offset: 0b1111_1111_1110_0000,
            },
        );

        assert_eq!(
            Instruction::new(0b1111_0000_1111_1111),
            Instruction::Trap {
                vec: 0b1111_1111,
            },
        );
    }
}
