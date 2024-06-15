pub mod engine;
pub mod flag;
pub mod memory_manager;
pub mod register;
pub mod variable_metadata;
pub mod line_processor;
pub mod command;
pub mod error_code;
pub mod status;
pub mod utils;

pub use engine::Engine;
pub use std::io;
use crate::error_code::ErrorCode;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibonacci() {
        let mut assembly: Engine;
        match Engine::new("./src/code_examples/fibonacci.txt") {
	        Ok(v) => assembly = v,
	        Err(e) => {
	            panic!("Could not run \"Fibonacci\" at ./src/code_examples/fibonacci.txt.\n{e}")
	        },
    	}
    	let result = assembly.execute(false);
    	match result {
	        Ok(_) => assert!(assembly.registers[0].get_word() == 89),
	        Err(e) => {
	            panic!("Errored during execution.\n{e}")
	        },
    	}
    }

    #[test]
    fn memory_char_manipulation() {
    	let mut assembly: Engine;
        match Engine::new("./src/code_examples/char_manipulation.txt") {
	        Ok(v) => assembly = v,
	        Err(e) => {
	            panic!("Could not run \"Char Manipulation\" at ./src/code_examples/char_manipulation.txt.\n{e}")
	        },
    	}
    	let result = assembly.execute(false);
    	match result {
	        Ok(_) => {
	        	let chars: Vec<u8> = assembly.get_memory(14);
	        	let string = "OHH THE MISERY".to_string();
	        	for i in 0..14 {
	        		let current_char = std::char::from_u32(chars[i].into());
	        		assert!(current_char.is_some());
	        		assert!(string.chars().skip(i).next().unwrap() == current_char.unwrap());
	        	}
	        }
	        Err(e) => {
	            panic!("Errored during execution.\n{e}")
	        },
    	}
    }
}