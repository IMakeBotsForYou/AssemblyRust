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
#[allow(unused_imports)]
use crate::{
	error_code::ErrorCode,
	utils::{
		initialize_engine,
		execute_engine,
		verify_memory
	}
};


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
				let ip = assembly.get_register_value("IP").expect("Couldn't get IP");
	            panic!("Errored during execution.\n{}\nLINE: {} ", e, ip)
	        },
    	}
    }

	#[test]
	fn memory_char_manipulation() {
		let mut assembly = initialize_engine("./src/code_examples/char_manipulation.txt");
		execute_engine(&mut assembly, false);
	
		let chars: Vec<u8> = assembly.get_memory(14);
		let expected_string = "OHH THE MISERY".as_bytes().to_vec();
		assert_eq!(chars, expected_string);
	}
	
	#[test]
	fn bubble_sort() {
		let mut assembly = initialize_engine("./src/code_examples/bubble_sort.txt");
		execute_engine(&mut assembly, false);
	
		let expected_sorted_array: Vec<u8> = vec![1, 1, 2, 4, 4, 8, 9, 37, 255];
		verify_memory(&assembly, &expected_sorted_array, 9);
	}
	
	#[test]
	fn find_factors() {
		let mut assembly = initialize_engine("./src/code_examples/find_factors.txt");
		execute_engine(&mut assembly, false);
	
		let expected_memory: Vec<u8> = vec![0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 17];
		verify_memory(&assembly, &expected_memory, 4 * 4);
	}


}