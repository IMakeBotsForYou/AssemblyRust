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

