use crate::register::Register;

pub struct LineProcessor {
    lines: Vec<String>,
    ip: usize
}

impl LineProcessor {
    pub fn new(lines: Vec<String>) -> Self {
        LineProcessor{
            lines: lines,
            ip: 0
        }
    }
    pub fn next(&mut self, ip_register: &mut Register, verbose: bool) -> Option<Vec<String>> {
        while self.ip < self.lines.len() {
            let line = self.lines[self.ip].clone();
            let whole_line = line.trim();

            // Split by ";" to remove comments and take the first part
            let without_comment: Vec<&str> = whole_line.split(';')
                .map(|s| s.trim())
                .collect();

            if without_comment[0] == "" {
                self.ip += 1;
                continue;
            } else {
                if verbose {
                    println!("[LINE] [{}] {}", self.ip, line);
                }

                let mut arguments: Vec<String> = without_comment[0]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                if arguments.len() > 1 {
                    let mut combined_arguments: Vec<String> = vec![arguments[0].clone()];
                    let further_arguments: Vec<String> = arguments[1..]
                        .join(" ")
                        .split(", ")
                        .map(|s| s.trim().to_string())
                        .collect();
                    combined_arguments.extend(further_arguments);
                    arguments = combined_arguments;
                }

                ip_register.load_word(self.ip as u16);
                self.ip += 1;
                return Some(arguments);
            }
        }
        None
    }
    pub fn set_ip(&mut self, value: usize) {
        self.ip = value;
    }
}

pub fn convert_vec_string_to_vec_str<'a>(vec: &'a [String]) -> Vec<&'a str> {
    vec.iter().map(|s| s.as_str()).collect()
}

