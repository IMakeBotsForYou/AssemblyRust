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

fn main() -> io::Result<()>  {
    // Initialize the engine
    let mut assembly: Engine;

    let file_path = std::env::args().skip(1).next().unwrap_or("code.txt".to_string());
    let _verbose  = std::env::args().skip(2).next().unwrap_or("false".to_string()).to_lowercase();
    let verbose = _verbose == "true".to_string() || _verbose == "t".to_string();
    match Engine::new(&file_path) {
        Ok(v) => assembly = v,
        Err(_) => {
            println!("Could not parse file.");
            return Ok(());
        },
    }

    // Execute the engine and handle any errors
    match assembly.execute(verbose) {
        Ok(()) => {
            println!("Execution completed successfully.");
        }
        Err(error) => {
            let ip = assembly.get_register_value("IP").unwrap_or_default();
            match error {
                ErrorCode::DivisionByZero => println!("Division By Zero error. Halted at {}", ip),
                ErrorCode::StackOverflow => println!("Stack Overflow error. Halted at {}", ip),
                ErrorCode::StackUnderflow => println!("Stack Underflow error. Halted at {}", ip),
                ErrorCode::InvalidOpcode => println!("Invalid Opcode. Halted at {}", ip),
                ErrorCode::InvalidRegister => println!("Invalid Register. Halted at {}", ip),
                ErrorCode::VariableAlreadyExists => println!("Attempted to save variable/label that already exists. Halted at {}", ip),
                ErrorCode::UnknownVariable => println!("Unknown Variable. Halted at {}", ip),
                ErrorCode::Overflow => println!("Overflow. Halted at {}", ip),
                ErrorCode::InvalidPointer(msg) => println!("Invalid Pointer. Halted at {}. {}", ip, msg),
                ErrorCode::NotEnoughSpace(msg) => println!("Not enough space to store variable. Halted at {}. {}", ip, msg),
                ErrorCode::InvalidValue(msg) => println!("Invalid value. Halted at {}. {}", ip, msg),
            }
        }
    }

    // Optionally, print out the registers to verify
    for register in &assembly.registers {
        let low_byte = (register.get_word() & 0xFF) as u8;
        let high_byte = ((register.get_word() >> 8) & 0xFF) as u8;
        println!("Register {}:\t{}\t({:08b} {:08b})", register.name, register.get_word(), high_byte, low_byte);
    }

    Ok(())
}