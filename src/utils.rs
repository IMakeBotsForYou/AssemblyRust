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
    let parsed_value = if value.starts_with("0x") {
        // Hexadecimal format
        usize::from_str_radix(&value[2..], 16).ok()
    } else if value.starts_with("0b") {
        // Binary format
        usize::from_str_radix(&value[2..], 2).ok()
    } else if value.starts_with("-0b") {
        // Negative Binary format
        let v = usize::from_str_radix(&value[3..], 2).ok()?;
        Some(isize::MAX as usize + 1 - v)
    } else if value.starts_with("-") {
        // Negative Decimal format
        let v = usize::from_str_radix(&value[1..], 10).ok()?;
        Some(isize::MAX as usize + 1 - v)
    } else {
        // Decimal format
        value.parse::<usize>().ok()
    };

    parsed_value
}

