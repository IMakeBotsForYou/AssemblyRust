// use std::io;
// use std::fs::File;
// use std::io::BufReader;
// use std::io::BufRead;

use crate::Engine;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

pub fn read_lines_from_file(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

pub fn parse_string_to_usize(value: &str) -> Option<u32> {
    let (radix, number) = if let Some(stripped) = value.strip_suffix('h') {
        // Hexadecimal format
        (16, stripped)
    } else if let Some(stripped) = value.strip_suffix('b') {
        // Binary format
        (2, stripped)
    } else {
        // Decimal format
        (10, value)
    };

    match i32::from_str_radix(number, radix) {
        Ok(v) => Some(v as u32),
        Err(_) => None
    }
}

pub fn initialize_engine(file_path: &str) -> Engine {
    match Engine::new(file_path) {
        Ok(engine) => engine,
        Err(e) => panic!("Could not run at {}.\n{}", file_path, e),
    }
}

pub fn execute_engine(assembly: &mut Engine, verbose: bool) {
    let result = assembly.execute(verbose);
    match result {
        Ok(_) => (),
        Err(e) => {
            let ip = assembly.lines.get_ip();
            panic!("Errored during execution.\n{}\nLINE: {}", e, ip);
        }
    }
}

pub fn verify_memory(assembly: &Engine, expected_memory: &[u8], length: usize) {
    let memory = assembly.get_memory(length);
    assert_eq!(memory, expected_memory);
}
