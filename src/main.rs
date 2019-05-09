use std::env;
use std::fs;

mod assembler;
mod instructions;
mod lc3;

fn main() {
    if let Err(err) = run() {
        println!("failed to run: {}", err)
    }
}

fn run() -> Result<(), String> {
    let os = include_str!("./os.asm");
    let os_executable = assembler::assemble("./os.asm", &os)?;
    lc3::Machine::new().run(&os_executable.instructions);

    let args: Vec<String> = env::args().collect();
    if let [_, filename] = args.as_slice() {
        let file = fs::read_to_string(filename).map_err(|e| format!("{}", e))?;
        let executable = assembler::assemble(filename, &file)?;
        lc3::Machine::new().run(&executable.instructions);
    }

    Ok(())
}
