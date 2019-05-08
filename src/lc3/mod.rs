use self::instructions::Instruction;

mod instructions;

pub struct Machine {
    /// addressable memory from 0x0000 -> 0xFFFF
    memory: [u16; 0xFFFF],
    /// general purpose registers
    regs: [u16; 8],
    /// program counter
    pc: u16,
    /// negative result condition code
    cc_neg: u16,
    /// positive result condition code
    cc_pos: u16,
    /// zero result condition code
    cc_zero: u16,
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            memory: [0; 0xFFFF],
            regs: [0; 8],
            pc: 0,
            cc_neg: 0,
            cc_pos: 0,
            cc_zero: 0,
        }
    }

    fn get_reg(&self, reg: u16) -> u16 {
        self.regs[reg as usize]
    }

    fn set_reg(&mut self, reg: u16, val: u16) {
        self.regs[reg as usize] = val;
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Add { dest, source_1, source_2, } => {
                let value = self.get_reg(source_1) + self.get_reg(source_2);
                self.set_reg(dest, value);
            }
            _ => {}
        }
    }

    pub fn run(&mut self, instructions: &[u16]) {
        let instructions = instructions
            .iter()
            .map(|instruction| Instruction::from(*instruction));
        for instruction in instructions {
            self.execute(instruction);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_instructions(machine: &mut Machine, instructions: Vec<Instruction>) {
        for instruction in instructions {
            machine.execute(instruction);
        }
    }

    fn from_regs(regs: [u16; 8]) -> Machine {
        Machine {
            memory: [0; 0xFFFF],
            regs,
            pc: 0,
            cc_neg: 0,
            cc_pos: 0,
            cc_zero: 0,
        }
    }

    #[test]
    fn test_run_machine() {
        let mut machine = from_regs([1, 2, 0, 0, 0, 0, 0, 0]);
        run_instructions(&mut machine, vec![
            Instruction::Add { dest: 0, source_1: 0, source_2: 1 },
        ]);
        assert_eq!(machine.regs[0], 3);
    }

}
