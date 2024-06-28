// use std::io;
// use std::fs::File;
// use std::io::BufReader;
// use std::io::BufRead;

use std::{
    io::{
        self,
        BufRead,
        BufReader
    },
    fs::File
};
use crate::Engine;

pub fn read_lines_from_file(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

pub fn parse_string_to_usize(value: &str) -> Option<u32> {
    let parsed_value = if value.ends_with("h") {
        // Hexadecimal format
        match i32::from_str_radix(&value[..value.len()-1], 16).ok() {
            Some(v) => Some(v as u32),
            None => None
        }
    } else if value.ends_with("b") {
        // Binary format
        match i32::from_str_radix(&value[..value.len()-1], 2).ok() {
            Some(v) => Some(v as u32),
            None => None
        }
    } else {
        // Decimal format
        match i32::from_str_radix(&value, 10).ok() {
            Some(v) => Some(v as u32),
            None => None
        }
    };
    parsed_value
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
        },
    }
}

pub fn verify_memory(assembly: &Engine, expected_memory: &[u8], length: usize) {
    let memory = assembly.get_memory(length);
    assert_eq!(memory, expected_memory);
}
