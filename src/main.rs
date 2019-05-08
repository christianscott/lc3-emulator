mod assembler;
mod lc3;

fn main() {
    let os = include_str!("./os.asm");
    let os_executable = assembler::assemble(&os);
    lc3::Machine::new().run(&os_executable.instructions);
}
