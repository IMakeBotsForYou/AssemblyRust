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

pub fn parse_string_to_usize(value: &str) -> Option<usize> {
    let parsed_value = if value.ends_with("h") {
        // Hexadecimal format
        isize::from_str_radix(&value[..value.len()-1], 16).ok()
    } else if value.ends_with("b") {
        // Binary format
        isize::from_str_radix(&value[..value.len()-1], 2).ok()
    } else {
        // Decimal format
        isize::from_str_radix(&value, 10).ok()
    };

    if let Some(v) = parsed_value {
        Some(v as usize)
    } else {
        None
    }
}

