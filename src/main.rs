use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::{HashSet, HashMap};


const MEMORY_SIZE: usize = 1024 * 16; // 16 KB
struct Register {
    value: u16,
    name: String,
}

// enum Command {
//     Mov,   
//     /*
//     Syntax
//     mov <reg>, <reg>
//     mov <reg>, <mem>
//     mov <mem>, <reg>
//     mov <reg>, <const>
//     mov <mem>, <const>
//     */
//     Push,
//     /*
//     Syntax
//     push <reg>
//     push <mem>
//     push <const>
//     */
//     Pop,
//     /*
//     Syntax
//     pop <reg>
//     pop <mem>
//     */
//     Lea,
//     /*
//     Syntax
//     lea <reg>, <mem>
//     */
//     Add,
//     /*
//     Syntax
//     add <reg>, <reg>
//     add <reg>, <mem>
//     add <reg>, <const>
//     add <mem>, <reg>
//     add <mem>, <const>
//     */
//     Sub,
//     /*
//     Syntax
//     sub <reg>, <reg>
//     sub <reg>, <mem>
//     sub <reg>, <const>
//     sub <mem>, <reg>
//     sub <mem>, <const>
//     */
//     Inc,
//     /*
//     Syntax
//     inc <reg>
//     inc <mem>
//     */
//     Dec,
//     /*
//     Syntax
//     dec <reg>
//     dec <mem>
//     */
//     Imul,
//     /*
//     Syntax
//     imul <reg>
//     imul <reg>, <reg>
//     imul <reg>, <mem>
//     imul <reg>, <const>
//     imul <reg>, <reg>, <const>
//     imul <reg>, <mem>, <const>
//     */
//     Idiv,
//     /*
//     Syntax
//     idiv <reg>
//     idiv <mem>
//     */
//     And,
//     /*
//     Syntax
//     and <reg>, <reg>
//     and <reg>, <mem>
//     and <reg>, <const>
//     and <mem>, <reg>
//     and <mem>, <const>
//     */
//     Or,
//     /*
//     Syntax
//     or <reg>, <reg>
//     or <reg>, <mem>
//     or <reg>, <const>
//     or <mem>, <reg>
//     or <mem>, <const>
//     */
//     Xor,
//     /*
//     Syntax
//     xor <reg>, <reg>
//     xor <reg>, <mem>
//     xor <reg>, <const>
//     xor <mem>, <reg>
//     xor <mem>, <const>
//     */
//     Not,
//     /*
//     Syntax
//     not <reg>
//     not <mem>
//     */
//     Neg,
//     /*
//     Syntax
//     neg <reg>
//     neg <mem>
//     */
//     Shl,
//     /*
//     Syntax
//     shl <reg>, <const>
//     shl <mem>, <const>
//     shl <reg>, <cl>
//     shl <mem>, <cl>
//     */
//     Shr,
//     /*
//     Syntax
//     shr <reg>, <const>
//     shr <mem>, <const>
//     shr <reg>, <cl>
//     shr <mem>, <cl>
//     */
//     Jmp,
//     /*
//     Syntax
//     jmp <label>
//     jmp <mem>
//     */
//     Je,
//     /*
//     Syntax
//     je <label>
//     */
//     Jne,
//     /*
//     Syntax
//     jne <label>
//     */
//     Jz,
//     /*
//     Syntax
//     jz <label>
//     */
//     Jg,
//     /*
//     Syntax
//     jg <label>
//     */
//     Jge,
//     /*
//     Syntax
//     jge <label>
//     */
//     Jle,
//     /*
//     Syntax
//     jle <label>
//     */
//     Cmp,
//     /*
//     Syntax
//     cmp <reg>, <reg>
//     cmp <reg>, <mem>
//     cmp <reg>, <const>
//     cmp <mem>, <reg>
//     cmp <mem>, <const>
//     */
//     Call,
//     /*
//     Syntax
//     call <label>
//     call <mem>
//     */
//     Ret,
//     /*
//     Syntax
//     ret
//     ret <const>
//     */
// }


fn read_lines_from_file(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}


#[derive(Debug)]
pub enum ErrorCode {
    DivisionByZero,
    StackOverflow(String),
    StackUnderflow,
    InvalidOpcode,
    UnknownVariable,
    InvalidPointer
    // Add more error codes as needed
}

#[derive(Debug)]
pub enum Status {
    Ok,
    Error(ErrorCode),
    Halted,
}

struct Engine {
    lines: Vec<String>, // lines of source code (.txt)
    registers: [Register; 8], // A-D, ESI, EDI, P
    memory_manager: MemoryManager, // 16 KB bytes of memory
    mode: bool, // false = reading data, true = reading code
    stack_pointer: usize, // pointer to the top of the stack within memory
    labels: Vec<usize>, // labels to jump to
    status: Status, // status: ok, error, halted,
    valid_registers: HashSet<String>,
}

impl Register {
    fn new(name: &str) -> Self {
        Self { value: 0, name: name.to_string() }
    }

    fn get_byte(&self, top: bool) -> u8 {
        if top {
            // return H
            (self.value >> 8) as u8
        } else {
            // return L
            (self.value & 0x00FF) as u8   
        }
    }

    fn get_word(&self) -> u16 {
        self.value
    }

    fn load_byte(&mut self, value: u8, top: bool) {
        if top {
            self.value = (self.value & 0x00FF) + ((value as u16) << 8);
        } else {
            self.value = (self.value & 0xFF00) + (value as u16);
        }
    }
    fn load_word(&mut self, value: u16) {
        self.value = value;
    }
}

fn get_register_size(reg_name: &str) -> Option<usize> {
    if reg_name.ends_with('X') {
        Some(16)
    } else if reg_name.ends_with('L') || reg_name.ends_with('H') {
        Some(8)
    } else {
        None
    }
}

// enum ValueSize {
//     Byte,
//     Word
// }

struct VariableMetadata {
    start_index: usize,
    length: usize,
}


struct MemoryManager {
    memory: Vec<u8>,
    variable_pointers: HashMap<String, VariableMetadata>,
}

impl MemoryManager {
    fn new(size: usize) -> Self {
        MemoryManager {
            memory: vec![0; size],
            variable_pointers: HashMap::new(),
        }
    }

    fn check_memory_address(&self, mem_address: usize) {
        if mem_address >= self.memory.len() {
            panic!("Invalid memory address");
        }
    }
    fn save_variable(&mut self, variable_name: String, data: &[u8]) -> Result<(), ErrorCode> {
        if let Ok(location) = self.find_free_block(data.len()) {
            // Copy data to the found location
            for (i, &byte) in data.iter().enumerate() {
                self.memory[location + i] = byte;
            }

            // Save the metadata
            self.variable_pointers.insert(variable_name, VariableMetadata {
                start_index: location,
                length: data.len(),
            });
            Ok(())
        } else {
            Err(ErrorCode::StackOverflow("Not enough contiguous free memory".to_string()))
        }
    }

    fn find_free_block(&self, length: usize) -> Result<usize, String> {
        let mut free_count = 0;
        let mut start_index = 0;

        for (i, &byte) in self.memory.iter().enumerate() {
            if byte == 0 {
                if free_count == 0 {
                    start_index = i;
                }
                free_count += 1;

                if free_count == length {
                    return Ok(start_index);
                }
            } else {
                free_count = 0;
            }
        }

        Err("Not enough contiguous free memory".to_string())
    }

    fn is_valid_array(&self, text: &str) -> Option<Vec<u8>> {
    // Adjust the regex pattern to correctly capture hexadecimal, binary, and decimal numbers
    let variable_pattern = Regex::new(r"^\[((?:&0x[0-9a-fA-F]+|&0b[01]+|\d+)(?:,(?:&0x[0-9a-fA-F]+|&0b[01]+|\d+))*)\]$").unwrap();
    if let Some(captures) = variable_pattern.captures(text) {
        if let Some(array_string) = captures.get(1) {

            let array_str = array_string.as_str();
            let elements: Vec<&str> = array_str.split(",").collect();
            let mut result = Vec::new();
            
            for element in elements {
                if element.starts_with("&0x") {
                    if let Ok(value) = u8::from_str_radix(&element[3..], 16) {
                        result.push(value);
                    } else {
                        return None;
                    }
                } else if element.starts_with("&0b") {
                    if let Ok(value) = u8::from_str_radix(&element[3..], 2) {
                        result.push(value);
                    } else {
                        return None;
                    }
                } else if let Ok(value) = element.parse::<u8>() {
                    result.push(value);
                } else {
                    return None;
                }
            }
            
            return Some(result);
        }
    }
    None
    }



    fn is_valid_variable_name(&self, text: &str) -> bool {
        let variable_pattern = Regex::new(r"^([a-zA-Z_]+)$").unwrap();
        if let Some(captures) = variable_pattern.captures(text) {
            if let Some(_) = captures.get(1) {
                return true;
            }
        }
        return false;
    }

    fn is_valid_pointer(&self, text: &str) -> Option<usize> {

        // Define the regex patterns
        let hex_pattern = Regex::new(r"^&0x([0-9a-fA-F]+)$").unwrap();
        let bin_pattern = Regex::new(r"^&0b([01]+)$").unwrap();
        let variable_pattern = Regex::new(r"^&([a-zA-Z_]+)$").unwrap();

        // Check if the input text matches the hex pattern
        if let Some(captures) = hex_pattern.captures(text) {
            if let Some(hex_str) = captures.get(1) {
                if let Ok(address) = usize::from_str_radix(hex_str.as_str(), 16) {
                    // Check if the address is within the valid range (16K)
                    if address < 1024 * 16 {
                        return Some(address)
                    } else {
                        return None
                    }
                }
            }
        }
    
        // Check if the input text matches the binary pattern
        if let Some(captures) = bin_pattern.captures(text) {
            if let Some(bin_str) = captures.get(1) {
                if let Ok(address) = usize::from_str_radix(bin_str.as_str(), 2) {
                    // Check if the address is within the valid range (16K)
                    if address < 1024 * 16 {
                        return Some(address)
                    } else {
                        return None
                    }
                }
            }
        }


        if let Some(captures) = variable_pattern.captures(text) {
            if let Some(variable_name) = captures.get(1) {
                if let Some(address) = self.variable_pointers.get(variable_name.as_str()) {
                    // Check if the address is within the valid range (16K)
                    if address.start_index < 1024 * 16 {
                        return Some(address.start_index)
                    } else {
                        return None
                    }
                }
            }
        }
    
        // If it doesn't match any pattern, return false
        None
    }

    fn get_variable_data(&self, variable_name: &str) -> Option<Vec<u8>> {
        if let Some(metadata) = self.variable_pointers.get(variable_name) {
            let start_index = metadata.start_index;
            Some(self._fetch_memory(start_index, metadata.length))
        }else {
            None
        }
    }

    // fn get_pointer_byte(&self, text: &str) -> Result<Vec<u8>, ErrorCode> {
    //     if let Some(pointer) = self.is_valid_pointer(text) {
    //         Ok(self._fetch_memory(pointer, 1))
    //     } else {
    //         Err(ErrorCode::InvalidPointer)
    //     }
    // }

    // fn get_pointer_word(&self, text: &str) -> Result<Vec<u8>, ErrorCode> {
    //     if let Some(pointer) = self.is_valid_pointer(text) {
    //         Ok(self._fetch_memory(pointer, 2))
    //     } else {
    //         Err(ErrorCode::InvalidPointer)
    //     }
    // }

    fn _fetch_memory(&self, start_index: usize, length: usize) -> Vec<u8>{
        self.memory[start_index..start_index+length].to_vec()
    }

}


fn parse_value_to_usize(value: &str) -> Option<usize> {
    if value.starts_with("0x") {
        // Hexadecimal format
        usize::from_str_radix(&value[2..], 16).ok()
    } else if value.starts_with("0b") {
        // Binary format
        usize::from_str_radix(&value[2..], 2).ok()
    } else {
        // Decimal format
        value.parse::<usize>().ok()
    }
}


fn get_register(name: &str) -> usize{
    match name {
        // "AX" => &mut self.registers[0],
        // "BX" => &mut self.registers[1],
        // "CX" => &mut self.registers[2],
        // "DX" => &mut self.registers[3],
        // "ESI" => &mut self.registers[4],
        // "DSI" => &mut self.registers[5],
        // "IP" => &mut self.registers[6],   
        // "FLAG" => &mut self.registers[7],   
        // _ => panic!("Invalid register name"),

        "AX"|"AL"|"AH" => 0,
        "BX"|"BL"|"BH" => 1,
        "CX"|"CL"|"CH" => 2,
        "DX"|"DL"|"DH" => 3,
        "ESI" => 4,
        "DSI" => 5,
        "IP" => 6,   
        "FLAG" => 7,   
        _ => panic!("Invalid register name"),
    }
}

impl Engine {
    fn new(file_name: &str) -> io::Result<Self> {
        let file_lines = read_lines_from_file(file_name)?;
        let my_registers: [Register; 8] = [
            Register::new("AX"),
            Register::new("BX"),
            Register::new("CX"),
            Register::new("DX"),
            Register::new("ESI"),
            Register::new("EDI"),
            Register::new("P"),
            Register::new("FLAG"),
        ];
        let valid_registers: HashSet<String> = [
            "AX", "BX", "CX", "DX", "ESI", "EDI", "P", "FLAG",
            "AL", "AH", "BL", "BH", "CL", "CH", "DL", "DH",
        ].iter().cloned().map(String::from).collect();
        Ok(Self {
            lines: file_lines,
            registers: my_registers,
            memory_manager: MemoryManager::new(MEMORY_SIZE),
            mode: false,
            stack_pointer: MEMORY_SIZE-1, // Initialize stack pointer to the end of memory
            labels: Vec::new(),
            status: Status::Ok,
            valid_registers
        })
    }
    fn is_valid_register(&self, reg_to_check: &str) -> bool {
        self.valid_registers.contains(reg_to_check)
    }

    fn both_valid_reg(&self, reg_1: &str, reg_2: &str) -> bool {
        self.is_valid_register(reg_1) && self.is_valid_register(reg_2)
    }



    fn get_variable(&self, text: &str) -> Option<Vec<u8>> {
        self.memory_manager.get_variable_data(text)
    }

    fn execute(&mut self) {
        let re = Regex::new(r#", | "#).unwrap();
        let mut ip = self.registers[6].value as usize;
        let mut line: String;

        while ip < self.lines.len() {
            // Split the string and collect into a vector of &str
            ip = self.registers[6].value as usize;
            line = self.lines[ip].clone();
            let arguments: Vec<&str> = re.split(&line)
                                        .filter(|&s| !s.is_empty())
                                        .collect();
            
            // Process the instruction based on the arguments
            match arguments.as_slice() {
                ["mov", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                    self.mov_reg_reg(reg_dst, reg_src);
                },

                ["mov", reg, variable] if self.is_valid_register(*reg) && self.get_variable(*variable).is_some() => {
                    if let Some(_) = self.get_variable(*variable) {
                        self.mov_reg_var(reg, variable);
                    }
                },

                ["mov", reg, constant] if self.is_valid_register(*reg) && parse_value_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        self.mov_reg_const(reg, value as u16);
                    }
                },

                ["mov", reg, mem_address] if self.is_valid_register(*reg) && self.memory_manager.is_valid_pointer(mem_address).is_some() => {
                   
                    if let Some(parsed_address) = self.memory_manager.is_valid_pointer(mem_address){
                        if self.memory_manager.is_valid_variable_name(&mem_address[1..]) {
                            self.mov_reg_const(reg, parsed_address.try_into().unwrap()); // pointers are always below 16386 (1024*16)
                        } else {
                            self.mov_reg_mem(reg, parsed_address);
                        }
                    }

                },

                ["mov", mem_address, reg] if self.is_valid_register(*reg) && self.memory_manager.is_valid_pointer(mem_address).is_some() => {
                    if let Some(parsed_address) = self.memory_manager.is_valid_pointer(mem_address){
                        self.mov_mem_reg(parsed_address, reg);
                    }
                },

                ////// ADD /////////
                ["add", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                    self.add_reg_reg(reg_dst, reg_src);
                },

                ["add", reg_dst, mem_address] if self.memory_manager.is_valid_pointer(mem_address).is_some() => {
                    if let Some(parsed_address) = self.memory_manager.is_valid_pointer(mem_address){
                        self.add_reg_mem(reg_dst, parsed_address);
                    }                
                },
                
                ["add", mem_address, reg_src] if self.memory_manager.is_valid_pointer(mem_address).is_some() => {
                    if let Some(parsed_address) = self.memory_manager.is_valid_pointer(mem_address){
                        self.add_mem_reg(parsed_address, reg_src);
                    }                
                },

                ["add", reg, constant] if self.is_valid_register(*reg) && parse_value_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        self.add_reg_const(reg, value as u16);
                    }
                },

                ["add", reg, variable] if self.is_valid_register(*reg) && self.get_variable(*variable).is_some() => {
                    if let Some(_) = self.get_variable(*variable) {
                        self.add_reg_variable(reg, variable);
                    }
                },

                ////////// SUB //////////
                ["sub", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                    self.sub_reg_reg(reg_dst, reg_src);
                },

                ["sub", reg_dst, mem_address] if self.memory_manager.is_valid_pointer(mem_address).is_some() => {
                    if let Some(parsed_address) = self.memory_manager.is_valid_pointer(mem_address){
                        self.sub_reg_mem(reg_dst, parsed_address);
                    }                
                },
                
                ["sub", mem_address, reg_src] if self.memory_manager.is_valid_pointer(mem_address).is_some() => {
                    if let Some(parsed_address) = self.memory_manager.is_valid_pointer(mem_address){
                        self.sub_mem_reg(parsed_address, reg_src);
                    }                
                },
                ["sub", reg, constant] if self.is_valid_register(*reg) && parse_value_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        self.sub_reg_const(reg, value as u16);
                    }
                },

                ["sub", reg, variable] if self.is_valid_register(*reg) && self.get_variable(*variable).is_some() => {
                    if let Some(_) = self.get_variable(*variable) {
                        self.sub_reg_variable(reg, variable);
                    }
                },

                ["let", variable_name, constant] if self.memory_manager.is_valid_variable_name(variable_name) &&
                (parse_value_to_usize(*constant).is_some() || self.memory_manager.is_valid_array(*constant).is_some() )=> {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        let _ = self.memory_manager.save_variable(variable_name.to_string(), &[value as u8]);
                    }
                    if let Some(array) = self.memory_manager.is_valid_array(*constant) {
                        let _ = self.memory_manager.save_variable(variable_name.to_string(), &array);
                    }
                }

                _ => {
                    println!("Unknown instruction: {}", arguments.join(", "));
                    // Handle unrecognized instructions
                },
            }

            
            // Increment the instruction pointer
            self.registers[get_register("IP")].value += 1;
            // self.get_register("IP").value += 1;
            ip = self.registers[6].value as usize;
        }
    }

    fn check_register_sizes(&self, dest: &str, src: &str) {
        let size_dest = get_register_size(dest);
        let size_src = get_register_size(src);

        if size_src != size_dest {
            panic!("Invalid Size");
        }
    }

    fn check_register_size_var(&self, dest: &str, var: &Vec<u8>)  {
        let size = match var.len() {
            1 => 'L',
            2 => 'X',
            _ => 'E',
        };

        if dest.chars().last() == Some('X') && size == 'L' {
            panic!("Invalid Size!");
        }
    }




    // MOV operations
    // REG <- Reg, Const, Var, Mem
    // Mem <- Reg, Const, Var, Mem
    // Var <- Reg, Const, Var, Mem
    // Const <- NOTHING, Const can't be moved into

    fn mov_reg_reg(&mut self, dest: &str, src: &str) {
        self.check_register_sizes(dest, src);

        match (src.chars().last(), dest.chars().last()) {
            (Some('L')|Some('H'), Some('L')|Some('H')) => {

                let top_src: bool = src.chars().last() == Some('H');
                let src_register_value = self.registers[get_register(src)].get_byte(top_src);

                let top_dst: bool = dest.chars().last() == Some('H');
                self.registers[get_register(dest)].load_byte(src_register_value, top_dst);

            },
            (Some('X'), Some('X')) => {
                let src_register_value = self.registers[get_register(src)].get_word();
                self.registers[get_register(dest)].load_word(src_register_value);
            }
            _ => {}
        }
    }

    fn mov_reg_const(&mut self, dest: &str, constant: u16) {
        match dest.chars().last() {
            Some('L') => self.registers[get_register(dest)].load_byte(constant as u8, false),
            Some('H') => self.registers[get_register(dest)].load_byte(constant as u8, true),
            Some('X') => self.registers[get_register(dest)].load_word(constant),
            _ => {}
        }
    }

    fn mov_reg_var(&mut self, dest: &str, src: &str) {
        if let Some(var) = self.get_variable(src) {
            self.check_register_size_var(dest, &var);

            match dest.chars().last() {

                Some('L')|Some('H') => {
                    self.registers[get_register(dest)].load_byte(var[0] as u8, dest.chars().last()==Some('H'));
                },

                Some('X') => {
                    self.registers[get_register(dest)].load_byte(var[0], true);
                    self.registers[get_register(dest)].load_byte(var[1], false);
                },

                _ => {}

            }
        }
    }

    fn mov_reg_mem(&mut self, dest: &str, mem_address: usize) {
        match dest.chars().last() {
            Some('L') => self.registers[get_register(dest)].load_byte(self.memory_manager.memory[mem_address], false),
            Some('H') => self.registers[get_register(dest)].load_byte(self.memory_manager.memory[mem_address], true),
            Some('X') => {
                self.memory_manager.check_memory_address(mem_address + 1);
                let result = ((self.memory_manager.memory[mem_address] as u16) << 8) + self.memory_manager.memory[mem_address + 1] as u16;
                self.registers[get_register(dest)].load_word(result);
            },
            _ => {}
        }
    }

    fn mov_mem_reg(&mut self, mem_address: usize, src: &str) {
        match src.chars().last() {
            Some('L') => self.memory_manager.memory[mem_address] = self.registers[get_register(src)].get_byte(false),
            Some('H') => self.memory_manager.memory[mem_address] = self.registers[get_register(src)].get_byte(true),
            Some('X') => {
                self.memory_manager.check_memory_address(mem_address + 1);
                self.memory_manager.memory[mem_address]     = self.registers[get_register(src)].get_byte(true);
                self.memory_manager.memory[mem_address + 1] = self.registers[get_register(src)].get_byte(false);
            },
            _ => {}
        }
    }

    fn add_reg_reg(&mut self, dest: &str, src: &str) {
        self.check_register_sizes(dest, src);

        let result: u16 = {
            match dest.chars().last() {
                Some('X') => {
                    self.registers[get_register(dest)].get_word().wrapping_add(self.registers[get_register(src)].get_word())
                },
                Some('L') => {
                    let dest_value_l = self.registers[get_register(dest)].get_byte(false);
                    let top = src.chars().last() == Some('H');
                    dest_value_l.wrapping_add(self.registers[get_register(src)].get_byte(top)).into()
                },
                Some('H') => {
                    let dest_value_h = self.registers[get_register(dest)].get_byte(true);
                    let top = src.chars().last() == Some('H');
                    dest_value_h.wrapping_add(self.registers[get_register(src)].get_byte(top)).into()
                }
                _ => {0},
            }
        };
        self.mov_reg_const(dest, result);
    }

    fn add_mem_reg(&mut self, mem_address: usize, src: &str) {
        match src.chars().last() {
            Some('L')|Some('H') => {
                let value = self.registers[get_register(src)].get_byte(src.chars().last()==Some('H'));
                self.memory_manager.memory[mem_address] = self.memory_manager.memory[mem_address].wrapping_add(value);
            },
            Some('X') => {
                self.memory_manager.check_memory_address(mem_address + 1);
                let h = self.registers[get_register(src)].get_byte(true);
                let l = self.registers[get_register(src)].get_byte(false);

                self.memory_manager.memory[mem_address] = self.memory_manager.memory[mem_address].wrapping_add(h);
                self.memory_manager.memory[mem_address + 1] = self.memory_manager.memory[mem_address + 1].wrapping_add(l);
            },
            _ => {}
        }
    }
    
    fn add_reg_mem(&mut self, dest: &str, mem_address: usize) {
        self.memory_manager.check_memory_address(mem_address);

        let result = {
                match dest.chars().last() {
                    Some('X') => {
                        let dest_value_x = self.registers[get_register(dest)].value;
                        let word_bytes = (self.memory_manager.memory[mem_address], self.memory_manager.memory[mem_address+1]);
                        dest_value_x.wrapping_add((((word_bytes.0 as u16)) << 8 ) + (word_bytes.1 as u16))
                    },
                    Some('L')|Some('H') => {
                        let top = dest.chars().last()==Some('H');
                        let dest_value_byte = self.registers[get_register(dest)].get_byte(top);
                        let byte = self.memory_manager.memory[mem_address];
                        dest_value_byte.wrapping_add(byte).into()
                    },
                    _ => {0}
                }
            };

        self.mov_reg_const(dest, result);
    }
    
    fn add_reg_const(&mut self, dest: &str, constant: u16) {
        
        let dest_value = self.registers[get_register(dest)].value;
        match dest.chars().last() {
            Some('X') => self.registers[get_register(dest)].load_word(dest_value.wrapping_add(constant)),
            Some('L')|Some('H') => {
                let top = dest.chars().last()==Some('H');
                let dest_value_l = self.registers[get_register(dest)].get_byte(top);
                self.registers[get_register(dest)].load_byte(dest_value_l.wrapping_add(constant as u8), top);
            },
            _ => {}
        };
    }

    fn add_reg_variable(&mut self, dest: &str, src: &str) {
        if let Some(var) = self.get_variable(src) {
            self.check_register_size_var(dest, &var);
            match dest.chars().last() {

                Some('L')|Some('H') => {
                    let reg_value = self.registers[get_register(dest)].get_byte(dest.chars().last()==Some('H'));
                    self.registers[get_register(dest)].load_byte(reg_value.wrapping_add(var[0]), dest.chars().last()==Some('H'));
                },

                Some('X') => {
                    let reg_value_l = self.registers[get_register(dest)].get_byte(false);
                    let reg_value_h = self.registers[get_register(dest)].get_byte(true);

                    self.registers[get_register(dest)].load_byte(reg_value_l.wrapping_add(var[0]), true);
                    self.registers[get_register(dest)].load_byte(reg_value_h.wrapping_add(var[1]), false);
                },

                _ => {}

            }
        }
    }
    
    //////////// SUB /////////////
    fn sub_reg_reg(&mut self, dest: &str, src: &str) {
        self.check_register_sizes(dest, src);

        let result: u16 = {
            match dest.chars().last() {
                Some('X') => {
                    self.registers[get_register(dest)].get_word().wrapping_sub(self.registers[get_register(src)].get_word())
                },
                Some('L') => {
                    let dest_value_l = self.registers[get_register(dest)].get_byte(false);
                    let top = src.chars().last() == Some('H');
                    dest_value_l.wrapping_sub(self.registers[get_register(src)].get_byte(top)).into()
                },
                Some('H') => {
                    let dest_value_h = self.registers[get_register(dest)].get_byte(true);
                    let top = src.chars().last() == Some('H');
                    dest_value_h.wrapping_sub(self.registers[get_register(src)].get_byte(top)).into()
                }
                _ => {0},
            }
        };
        self.mov_reg_const(dest, result);
    }

    fn sub_mem_reg(&mut self, mem_address: usize, src: &str) {
        match src.chars().last() {
            Some('L')|Some('H') => {
                let value = self.registers[get_register(src)].get_byte(src.chars().last()==Some('H'));
                self.memory_manager.memory[mem_address] = self.memory_manager.memory[mem_address].wrapping_sub(value);
            },
            Some('X') => {
                self.memory_manager.check_memory_address(mem_address + 1);
                let h = self.registers[get_register(src)].get_byte(true);
                let l = self.registers[get_register(src)].get_byte(false);

                self.memory_manager.memory[mem_address] = self.memory_manager.memory[mem_address].wrapping_sub(h);
                self.memory_manager.memory[mem_address + 1] = self.memory_manager.memory[mem_address + 1].wrapping_sub(l);
            },
            _ => {}
        }
    }
    
    fn sub_reg_mem(&mut self, dest: &str, mem_address: usize) {
        self.memory_manager.check_memory_address(mem_address);

        let result = {
                match dest.chars().last() {
                    Some('X') => {
                        let dest_value_x = self.registers[get_register(dest)].value;
                        let word_bytes = (self.memory_manager.memory[mem_address], self.memory_manager.memory[mem_address+1]);
                        dest_value_x.wrapping_sub((((word_bytes.0 as u16)) << 8 ) + (word_bytes.1 as u16))
                    },
                    Some('L')|Some('H') => {
                        let top = dest.chars().last()==Some('H');
                        let dest_value_l = self.registers[get_register(dest)].get_byte(top);
                        let byte = self.memory_manager.memory[mem_address];
                        dest_value_l.wrapping_sub(byte).into()
                    },
                    _ => {0}
                }
            };

        self.mov_reg_const(dest, result);
    }
    
    fn sub_reg_const(&mut self, dest: &str, constant: u16) {
        let dest_value = self.registers[get_register(dest)].value;
        match dest.chars().last() {
            Some('X') => {self.registers[get_register(dest)].load_word(dest_value.wrapping_sub(constant));},
            Some('L')|Some('H') => {
                let top = dest.chars().last()==Some('H');
                let dest_value_l = self.registers[get_register(dest)].get_byte(top);
                self.registers[get_register(dest)].load_byte(dest_value_l.wrapping_sub(constant as u8), top);
            },
            _ => {}
        }

    }

    fn sub_reg_variable(&mut self, dest: &str, src: &str) {
        if let Some(var) = self.get_variable(src) {
            self.check_register_size_var(dest, &var);
            match dest.chars().last() {

                Some('L')|Some('H') => {
                    let reg_value = self.registers[get_register(dest)].get_byte(dest.chars().last()==Some('H'));
                    self.registers[get_register(dest)].load_byte(reg_value.wrapping_sub(var[0]), dest.chars().last()==Some('H'));
                },

                Some('X') => {
                    let reg_value_l = self.registers[get_register(dest)].get_byte(false);
                    let reg_value_h = self.registers[get_register(dest)].get_byte(true);

                    self.registers[get_register(dest)].load_byte(reg_value_l.wrapping_sub(var[0]), true);
                    self.registers[get_register(dest)].load_byte(reg_value_h.wrapping_sub(var[1]), false);
                },

                _ => {}

            }
        }
    }
    // fn div_reg_reg(&mut self, src: &str) {
    //     self.check_register_sizes("AX", src);
    
    //     let src_value = self.get_register_value(src);
    //     let ax_value = self.get_register_value("AX");
    
    //     if src_value != 0 {
    //         let quotient = ax_value / src_value;
    //         let remainder = ax_value % src_value;
    
    //         self.load_into_register("CX", quotient);
    //         self.load_into_register("DX", remainder);
    //     } else {
    //         // Handle division by zero error
    //         panic!("Division by zero error");
    //     }
    // }
    
    // fn div_reg_mem(&mut self, mem_address: usize) {
    //     self.memory_manager.check_memory_address(mem_address);
        
    //     let mem_value = self.memory_manager.memory[mem_address] as u16;
    //     let ax_value = self.get_register_value("AX");
    
    //     if mem_value != 0 {
    //         let quotient = ax_value / mem_value;
    //         let remainder = ax_value % mem_value;
    
    //         self.load_into_register("CX", quotient);
    //         self.load_into_register("DX", remainder);
    //     } else {
    //         // Handle division by zero error
    //         panic!("Division by zero error");
    //     }
    // }
    
    // fn div_reg_const(&mut self, constant: u16) {
    //     let ax_value = self.get_register_value("AX");
    
    //     if constant != 0 {
    //         let quotient = ax_value / constant;
    //         let remainder = ax_value % constant;
    
    //         self.load_into_register("CX", quotient);
    //         self.load_into_register("DX", remainder);
    //     } else {
    //         // Handle division by zero error
    //         panic!("Division by zero error");
    //     }
    // }
    
    // fn mul_reg_reg(&mut self, src: &str) {
    //     self.check_register_sizes("AX", src);
    
    //     let src_value = self.get_register_value(src);
    //     let ax_value = self.get_register_value("AX");
    
    //     let result = ax_value.wrapping_mul(src_value);
    //     self.load_into_register("BX", result);
    // }
    
    // fn mul_reg_mem(&mut self, mem_address: usize) {
    //     self.memory_manager.check_memory_address(mem_address);
    
    //     let mem_value = self.memory_manager.memory[mem_address] as u16;
    //     let ax_value = self.get_register_value("AX");
    
    //     let result = ax_value.wrapping_mul(mem_value);
    //     self.load_into_register("BX", result);
    // }
    
    // fn mul_reg_const(&mut self, constant: u16) {
    //     let ax_value = self.get_register_value("AX");
    
    //     let result = ax_value.wrapping_mul(constant);
    //     self.load_into_register("BX", result);
    // }
}


fn main() -> io::Result<()> {
    let mut assembly: Engine = Engine::new("code.txt")?;

    // Print out the lines to verify
    for line in &assembly.lines {
        println!("{}", line);
    }
    assembly.execute();

    // Optionally, print out the registers to verify
    for register in &assembly.registers {
        println!("Register {}: {}", register.name, register.value);
    }
    Ok(())
}

