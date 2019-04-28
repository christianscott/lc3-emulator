mod assembler;
mod lc3;

fn main() {
  let os = include_str!("./os.asm");
  let os_executable = assembler::assemble(String::from(os));
  let instructions = os_executable
    .instructions
    .iter()
    .map(|instruction| lc3::Instruction::from(*instruction));
  lc3::Machine::run(instructions.collect());
}
