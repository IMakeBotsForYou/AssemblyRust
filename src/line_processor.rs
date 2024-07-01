use crate::register::Register;

pub struct LineProcessor {
    lines: Vec<String>,
    ip: usize,
}

impl LineProcessor {
    pub fn new(lines: Vec<String>) -> Self {
        LineProcessor { lines, ip: 0 }
    }

    pub fn next_line(&mut self) -> Option<Vec<String>> {
        while self.ip < self.lines.len() {
            let line = self.lines[self.ip].clone();
            let whole_line = line.trim();

            // Split by ";" to remove comments and take the first part
            let without_comment = whole_line.split(';').next().unwrap_or("").trim();

            if without_comment.is_empty() {
                self.ip += 1;
                continue;
            }

            let instruction = without_comment
                .split_whitespace()
                .next()
                .unwrap_or("NOP")
                .to_string();

            if instruction == "NOP" {
                self.ip += 1;
                continue;
            }

            let operands: Vec<String> = without_comment[instruction.len()..]
                .split(',')
                .map(str::trim)
                .map(String::from)
                .collect();

            self.ip += 1;
            let mut parts = vec![instruction];
            if operands[0] != *"" {
                parts.extend(operands);
            }
            return Some(parts);
        }
        None
    }

    pub fn set_ip(&mut self, value: usize) {
        self.ip = value;
    }
    pub fn get_ip(&self) -> usize {
        self.ip
    }
    pub fn update_ip_register(&mut self, ip_register: &mut Register) {
        ip_register.load_word(self.ip as u16);
    }

    pub fn peak(&mut self) -> Option<(Vec<String>, usize)> {
        let prev_ip = self.ip; // Capture current instruction pointer
        let res = self.next_line(); // Get the next instruction
        let next_ip = self.ip; // Capture the updated instruction pointer
        self.set_ip(prev_ip); // Restore the instruction pointer to its original state
        res.map(|line| (line, next_ip)) // Return the result along with the updated instruction pointer
    }
}
