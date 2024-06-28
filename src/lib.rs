pub mod engine;
pub mod flag;
pub mod memory_manager;
pub mod register;
pub mod variable_metadata;
pub mod line_processor;
pub mod instruction;
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
    fn add_sub() {
        let mut assembly = initialize_engine("./src/unit_tests/add_sub.txt");
        execute_engine(&mut assembly, false);
        assert!(assembly.registers[0].get_word() == 512); // AX
        assert!(assembly.registers[1].get_word() == 513); // BX
        assert!(assembly.registers[2].get_word() == 1);   // CX
    }
    #[test]
    fn mul_div() {
        let mut assembly = initialize_engine("./src/unit_tests/mul_div.txt");
        execute_engine(&mut assembly, false);
        assert!(assembly.registers[0].get_word() == 3);  // AX
        assert!(assembly.registers[1].get_word() == 3);  // BX
        assert!(assembly.registers[2].get_word() == 0);  // CX
        assert!(assembly.registers[3].get_word() == 1);  // DX
        assert!(assembly.registers[4].get_word() == 10); // SI
        assert!(assembly.registers[5].get_word() == 5);  // DI
    }
    #[test]
    fn imul_idiv() {
        let mut assembly = initialize_engine("./src/unit_tests/imul_idiv.txt");
        execute_engine(&mut assembly, false);
        assert!(assembly.registers[0].get_word() as i16 == -5);  // AX
        assert!(assembly.registers[1].get_word() as i16 == -5);  // BX
        assert!(assembly.registers[2].get_word() == 2);  // CX
        assert!(assembly.registers[3].get_word() == 0);  // DX
    }
    #[test]
    fn shr_shl() {
        let mut assembly = initialize_engine("./src/unit_tests/shr_shl.txt");
        execute_engine(&mut assembly, false);
    
        let expected_memory: Vec<u8> = vec![4, 0, 0, 0];
        verify_memory(&assembly, &expected_memory, 4);
        assert!(assembly.registers[1].get_word() == 5);
    }
    #[test]
    fn fibonacci() {
        let mut assembly = initialize_engine("./src/code_examples/fibonacci.txt");
        execute_engine(&mut assembly, false);
        assert!(assembly.registers[0].get_word() == 89);
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
