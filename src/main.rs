//! CLI tool for Assembler

use assembly::io;

use assembly::{execute_engine, initialize_engine};

fn main() -> io::Result<()> {
    let mut args = std::env::args();
    args.next().unwrap();
    // Initialize the engine

    // Arg 1 is file_path
    let file_path = match args.next() {
        Some(file_path) => file_path,
        None => {
            // Not sure what to do here and if it was intended but now I guess there is a spot to write any warnings you`d like
            println!("It is best to provide a file path going to default");
            String::from("./examples/code.asm")
        }
    };

    // Arg 2 is debug
    let debug = match args.next() {
        Some(input) => ["true", "t"].contains(&input.to_lowercase().as_str()),
        None => false,
    };

    let mut engine = initialize_engine(&file_path);
    execute_engine(&mut engine, debug);
    if !debug {
        // already printing every time.
        println!("{}", engine);
    }
    Ok(())
}
