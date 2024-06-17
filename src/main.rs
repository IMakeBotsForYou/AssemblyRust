use assembly::engine::Engine;  // Adjust the path based on your crate name and module structure
use assembly::error_code::ErrorCode;
use assembly::io;

#[allow(dead_code)]
mod engine;
#[allow(dead_code)]
mod flag;
#[allow(dead_code)]
mod memory_manager;
#[allow(dead_code)]
mod register;
#[allow(dead_code)]
mod variable_metadata;
#[allow(dead_code)]
mod line_processor;
#[allow(dead_code)]
mod command;
#[allow(dead_code)]
mod error_code;
#[allow(dead_code)]
mod status;
#[allow(dead_code)]
mod utils;
// #[allow(dead_code)]
// mod compiler;

#[allow(unused_imports)]
use crate::utils::{
            initialize_engine,
            execute_engine,
            verify_memory
	     };



fn main() -> io::Result<()>  {
    // Initialize the engine

    let file_path = std::env::args().skip(1).next().unwrap_or("code.txt".to_string());
    let _verbose  = std::env::args().skip(2).next().unwrap_or("false".to_string()).to_lowercase();
    let verbose = _verbose == "true".to_string() || _verbose == "t".to_string();

    let mut assembly = initialize_engine(&file_path);
    execute_engine(&mut assembly, verbose);

    // Optionally, print out the registers to verify
    for register in &assembly.registers {
        let low_byte = (register.get_word() & 0xFF) as u8;
        let high_byte = ((register.get_word() >> 8) & 0xFF) as u8;
        println!("Register {}:\t{}\t({:08b} {:08b})", register.name, register.get_word(), high_byte, low_byte);
    }

    Ok(())
}