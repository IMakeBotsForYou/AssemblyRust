use assembly::engine::Engine;  // Adjust the path based on your crate name and module structure
use assembly::error_code::ErrorCode;
use assembly::io;

mod engine;
mod flag;
mod memory_manager;
mod register;
mod variable_metadata;
mod line_processor;
mod command;
mod error_code;
mod status;
mod utils;

fn main() -> io::Result<()>  {
    // Initialize the engine
    let mut assembly: Engine;

    match Engine::new("code.txt") {
        Ok(v) => assembly = v,
        Err(_) => {
            println!("Could not parse file.");
            return Ok(());
        },
    }

    // Execute the engine and handle any errors
    match assembly.execute() {
        Ok(()) => {
            println!("Execution completed successfully.");
        }
        Err(error) => {
            match error {
                ErrorCode::DivisionByZero => println!("Division By Zero error. Halted at {}", assembly.get_register_value("IP")),
                ErrorCode::StackOverflow => println!("Stack Overflow error. Halted at {}", assembly.get_register_value("IP")),
                ErrorCode::StackUnderflow => println!("Stack Underflow error. Halted at {}", assembly.get_register_value("IP")),
                ErrorCode::InvalidOpcode => println!("Invalid Opcode. Halted at {}", assembly.get_register_value("IP")),
                ErrorCode::InvalidRegister => println!("Invalid Register. Halted at {}", assembly.get_register_value("IP")),
                ErrorCode::VariableAlreadyExists => println!("Attempted to save variable/label that already exists. Halted at {}", assembly.get_register_value("IP")),
                ErrorCode::UnknownVariable => println!("Unknown Variable. Halted at {}", assembly.get_register_value("IP")),
                ErrorCode::Overflow => println!("Overflow. Halted at {}", assembly.get_register_value("IP")),
                ErrorCode::InvalidPointer(msg) => println!("Invalid Pointer. Halted at {}. {}", assembly.get_register_value("IP"), msg),
                ErrorCode::NotEnoughSpace(msg) => println!("Not enough space to store variable. Halted at {}. {}", assembly.get_register_value("IP"), msg),
                ErrorCode::InvalidValue(msg) => println!("Invalid value. Halted at {}. {}", assembly.get_register_value("IP"), msg),
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