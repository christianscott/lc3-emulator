#![allow(dead_code)]

/// 2^16 locations, each containing one word (16 bits).
/// Numbered from 0x0000 -> 0xFFFF
const MEMORY_ADDRESS_SPACE: [u16; 0xFFFF] = [0; 0xFFFF];

const ADD_OPCODE: u8 = 0b0001;
const AND: u8 = 0b0101;
const BR: u8 = 0b0000;
const JMP: u8 = 0b1100;
const JSR: u8 = 0b0100;
const LD: u8 = 0b0010;
const LDI: u8 = 0b1010;
const LDR: u8 = 0b0110;
const LEA: u8 = 0b1110;
const NOT: u8 = 0b1001;
const RET: u8 = 0b1100;
const RTI: u8 = 0b1000;
const ST: u8 = 0b0011;
const STI: u8 = 0b1011;
const STR: u8 = 0b0111;
const TRAP: u8 = 0b1111;
const ILLEGAL: u8 = 0b1101;

#[derive(Debug, PartialEq)]
enum Instruction {
    Add {
        dest: u8,
        source_1: u8,
        source_2: u8,
    },
    AddImmediate {
        dest: u8,
        source: u8,
        value: u16,
    },
    AND,
    BR,
    JMP,
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

fn opcode_is(instruction: u16, opcode: u8) -> bool {
    slice_bits(instruction, 15, 12) == opcode
}

// indices are from 15 (leftmost) to 0 (rightmost):
// [15|14|13|12|11|10|09|08|07|06|05|04|03|02|01|00]
fn slice_bits(instruction: u16, from: u8, to: u8) -> u8 {
    let slice_size = from - to + 1;
    let mask = (1 << slice_size) - 1;
    ((instruction >> to) & mask) as u8
}

fn set(instruction: u16, bit: u16) -> bool {
    instruction & (1 << bit) == (1 << bit)
}

/// expects a 5 bit number
fn sign_extend(x: u8) -> u16 {
    assert!(x < 0b100000, "must be a 5 bit number");
    if set(x as u16, 4) {
        (x as u16) | 0b1111111111100000
    } else {
        println!("{:#b}", x );
        x as u16
    }
}

impl Instruction {
    fn new(instruction: u16) -> Instruction {
        if opcode_is(instruction, ADD_OPCODE) {
            if set(instruction, 5) {
                Instruction::AddImmediate {
                    dest: slice_bits(instruction, 11, 9),
                    source: slice_bits(instruction, 8, 6),
                    value: sign_extend(slice_bits(instruction, 4, 0)),
                }
            } else {
                Instruction::Add {
                    dest: slice_bits(instruction, 11, 9),
                    source_1: slice_bits(instruction, 8, 6),
                    source_2: slice_bits(instruction, 2, 0),
                }
            }
        } else {
            Instruction::ILLEGAL
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
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b10001), 0b1111111111110001);
        assert_eq!(sign_extend(0b1001), 0b1001);
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
            Instruction::new(0b0001_100_010_1_01001),
            Instruction::AddImmediate {
                dest: 0b100,
                source: 0b010,
                value: 0b1001,
            }
        );
    }
}
