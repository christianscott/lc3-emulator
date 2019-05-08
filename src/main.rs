mod assembler;
mod lc3;

fn main() {
    let os = include_str!("./os.asm");
    match assembler::assemble("./os.asm", &os) {
        Ok(os_executable) => lc3::Machine::new().run(&os_executable.instructions),
        Err(err) => println!("failed to assemble: {}", err),
    }
}
