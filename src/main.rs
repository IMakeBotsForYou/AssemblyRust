use regex::Regex;
use std::{
    fs::File,
    io:: {
        self, BufRead, BufReader
    },
    collections::{
        HashSet, HashMap
    },
    str::FromStr,
};
// use std::io::{self, BufRead, BufReader};
// use std::collections::{HashSet, HashMap};
// use std::str::FromStr;


const MEMORY_SIZE: usize = 1024 * 16; // 16 KB
struct Register {
    value: u16,
    name: String,
}

enum Command {
    Mov,   
    /* Move data
    Syntax
    mov <reg>, <reg>
    mov <reg>, <mem>
    mov <mem>, <reg>
    mov <reg>, <const>
    mov <mem>, <const>
    */
    Push,
    /* Push to stack
    Syntax
    push <reg>
    push <mem>
    push <const>
    */
    Pop,
    /* Pop from stack
    Syntax
    pop <reg>
    pop <mem>
    */
    Lea,
    /* 
    Syntax
    lea <reg>, <mem>
    */
    Add,
    /*
    Syntax
    add <reg>, <reg>
    add <reg>, <mem>
    add <reg>, <const>
    add <mem>, <reg>
    add <mem>, <const>
    */
    Sub,
    /*
    Syntax
    sub <reg>, <reg>
    sub <reg>, <mem>
    sub <reg>, <const>
    sub <mem>, <reg>
    sub <mem>, <const>
    */
    Inc,
    /*
    Syntax
    inc <reg>
    inc <mem>
    */
    Dec,
    /*
    Syntax
    dec <reg>
    dec <mem>
    */
    Mul,
    /*
    Syntax
    mul <reg>
    mul <mem>
    mul <const>
    mul <var>
    */
    Div,
    /*
    Syntax
    div <reg>
    div <mem>
    div <var>
    div <const>
    */
    And,
    /*
    Syntax
    and <reg>, <reg>
    and <reg>, <mem>
    and <reg>, <const>
    and <mem>, <reg>
    and <mem>, <const>
    */
    Or,
    /*
    Syntax
    or <reg>, <reg>
    or <reg>, <mem>
    or <reg>, <const>
    or <mem>, <reg>
    or <mem>, <const>
    */
    Xor,
    /*
    Syntax
    xor <reg>, <reg>
    xor <reg>, <mem>
    xor <reg>, <const>
    xor <mem>, <reg>
    xor <mem>, <const>
    */
    Not,
    /*
    Syntax
    not <reg>
    not <mem>
    */
    Neg,
    /*
    Syntax
    neg <reg>
    neg <mem>
    */
    Shl,
    /*
    Syntax
    shl <reg>, <const>
    shl <mem>, <const>
    shl <reg>, <cl>
    shl <mem>, <cl>
    */
    Shr,
    /*
    Syntax
    shr <reg>, <const>
    shr <mem>, <const>
    shr <reg>, <cl>
    shr <mem>, <cl>
    */
    Jmp,
    /*
    Syntax
    jmp <label>
    jmp <mem>
    */
    Je,
    /*
    Syntax
    je <label>
    */
    Jne,
    /*
    Syntax
    jne <label>
    */
    Jz,
    /*
    Syntax
    jz <label>
    */
    Jg,
    /*
    Syntax
    jg <label>
    */
    Jge,
    /*
    Syntax
    jge <label>
    */
    Jle,
    /*
    Syntax
    jle <label>
    */
    Cmp,
    /*
    Syntax
    cmp <reg>, <reg>
    cmp <reg>, <mem>
    cmp <reg>, <const>
    cmp <mem>, <reg>
    cmp <mem>, <const>
    */
    Call,
    /*
    Syntax
    call <label>
    call <mem>
    */
    Ret,
    /*
    Syntax
    ret
    ret <const>
    */
}

impl Command {
    fn get_help_string(command: Command) -> String {
        match command {
            Command::Mov => {
                "The 'mov' command moves data between registers or between memory and registers.
Syntax:
    mov <reg>, <reg>
    mov <reg>, <mem>
    mov <mem>, <reg>
    mov <reg>, <const>
    mov <mem>, <const>
    mov <reg>, <var>".to_string()
            },
            Command::Push => {
                "The 'push' command pushes a value onto the stack.
Syntax:
    push <reg>
    push <mem>
    push <const>
    push <var>".to_string()
            },
            Command::Pop => {
                "The 'pop' command pops a value from the stack.
Syntax:
    pop <reg>
    pop <mem>".to_string()
            },
            Command::Lea => {
                "The 'lea' command loads the effective address of the operand into a register.
Syntax:
    lea <reg>, <mem>".to_string()
            },
            Command::Add => {
                "The 'add' command adds two operands.
Syntax:
    add <reg>, <reg>
    add <reg>, <mem>
    add <reg>, <const>
    add <mem>, <reg>
    add <mem>, <const>".to_string()
            },
            Command::Sub => {
                "The 'sub' command subtracts the second operand from the first.
Syntax:
    sub <reg>, <reg>
    sub <reg>, <mem>
    sub <reg>, <const>
    sub <mem>, <reg>
    sub <mem>, <const>".to_string()
            },
            Command::Inc => {
                "The 'inc' command increments an operand by one.
Syntax:
    inc <reg>
    inc <mem>".to_string()
            },
            Command::Dec => {
                "The 'dec' command decrements an operand by one.
Syntax:
    dec <reg>
    dec <mem>".to_string()
            },
            Command::Mul => {
                "The 'mul' command multiplies the operand by the accumulator.
Syntax:
    mul <reg>
    mul <mem>
    mul <const>
    mul <var>".to_string()
            },
            Command::Div => {
                "The 'div' command divides the accumulator by the operand.
Syntax:
    div <reg>
    div <mem>
    div <var>
    div <const>".to_string()
            },
            Command::And => {
                "The 'and' command performs a bitwise AND operation.
Syntax:
    and <reg>, <reg>
    and <reg>, <mem>
    and <reg>, <const>
    and <mem>, <reg>
    and <mem>, <const>".to_string()
            },
            Command::Or => {
                "The 'or' command performs a bitwise OR operation.
Syntax:
    or <reg>, <reg>
    or <reg>, <mem>
    or <reg>, <const>
    or <mem>, <reg>
    or <mem>, <const>".to_string()
            },
            Command::Xor => {
                "The 'xor' command performs a bitwise exclusive OR operation.
Syntax:
    xor <reg>, <reg>
    xor <reg>, <mem>
    xor <reg>, <const>
    xor <mem>, <reg>
    xor <mem>, <const>".to_string()
            },
            Command::Not => {
                "The 'not' command performs a bitwise NOT operation.
Syntax:
    not <reg>
    not <mem>".to_string()
            },
            Command::Neg => {
                "The 'neg' command negates the operand, creating its two's complement.
Syntax:
    neg <reg>
    neg <mem>".to_string()
            },
            Command::Shl => {
                "The 'shl' command shifts the bits of the operand to the left.
Syntax:
    shl <reg>, <const>
    shl <mem>, <const>
    shl <reg>, <cl>
    shl <mem>, <cl>".to_string()
            },
            Command::Shr => {
                "The 'shr' command shifts the bits of the operand to the right.
Syntax:
    shr <reg>, <const>
    shr <mem>, <const>
    shr <reg>, <cl>
    shr <mem>, <cl>".to_string()
            },
            Command::Jmp => {
                "The 'jmp' command jumps to the specified label or memory location.
Syntax:
    jmp <label>
    jmp <mem>".to_string()
            },
            Command::Je => {
                "The 'je' command jumps to the specified label if the zero flag is set.
Syntax:
    je <label>".to_string()
            },
            Command::Jne => {
                "The 'jne' command jumps to the specified label if the zero flag is not set.
Syntax:
    jne <label>".to_string()
            },
            Command::Jz => {
                "The 'jz' command jumps to the specified label if the zero flag is set (alias for 'je').
Syntax:
    jz <label>".to_string()
                            },
            Command::Jg => {
                "The 'jg' command jumps to the specified label if the greater flag is set.
Syntax:
    jg <label>".to_string()
            },
            Command::Jge => {
                "The 'jge' command jumps to the specified label if the greater or equal flag is set.
Syntax:
    jge <label>".to_string()
            },
            Command::Jle => {
                "The 'jle' command jumps to the specified label if the less or equal flag is set.
    Syntax:
        jle <label>".to_string()
            },
            Command::Cmp => {
                "The 'cmp' command compares two operands.
Syntax:
    cmp <reg>, <reg>
    cmp <reg>, <mem>
    cmp <reg>, <const>
    cmp <mem>, <reg>
    cmp <mem>, <const>".to_string()
            },
            Command::Call => {
                "The 'call' command calls a procedure at the specified label or memory location.
Syntax:
    call <label>
    call <mem>".to_string()
            },
            Command::Ret => {
                "The 'ret' command returns from a procedure.
Syntax:
    ret
    ret <const>".to_string()
            },
        }
    }
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Command, ()> {
        match s.to_lowercase().as_str() {
            "mov" => Ok(Command::Mov),
            "push" => Ok(Command::Push),
            "pop" => Ok(Command::Pop),
            "add" => Ok(Command::Add),
            "sub" => Ok(Command::Sub),
            "lea" => Ok(Command::Lea),
            "inc" => Ok(Command::Inc),
            "dec" => Ok(Command::Dec),
            "mul" => Ok(Command::Mul),
            "div" => Ok(Command::Div),
            "and" => Ok(Command::And),
            "or" => Ok(Command::Or),
            "xor" => Ok(Command::Xor),
            "not" => Ok(Command::Not),
            "neg" => Ok(Command::Neg),
            "shl" => Ok(Command::Shl),
            "shr" => Ok(Command::Shr),
            "jmp" => Ok(Command::Jmp),
            "je" => Ok(Command::Je),
            "jne" => Ok(Command::Jne),
            "jz" => Ok(Command::Jz),
            "jg" => Ok(Command::Jg),
            "jge" => Ok(Command::Jge),
            "jle" => Ok(Command::Jle),
            "cmp" => Ok(Command::Cmp),
            "call" => Ok(Command::Call),
            "ret" => Ok(Command::Ret),
            _ => Err(()),
        }
    }
}


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
    InvalidPointer(String),
    NotEnoughSpace(String),
    // Add more error codes as needed
}

#[derive(Debug)]
pub enum Status {
    Ok,
    Error(ErrorCode),
    Halted,
}

#[derive(Debug, Clone, Copy)]
enum Flag {
    // Carry = 0b0001,        // Carry Flag
    Parity = 0b0010,       // Parity Flag
    // AuxiliaryCarry = 0b0100, // Auxiliary Carry Flag
    Zero = 0b1000,         // Zero Flag
    Sign = 0b0001_0000,         // Sign Flag
    Overflow = 0b0010_0000,  // Overflow Flag
}

impl Flag {
    // Function to return the flag value
    fn value(&self) -> u16 {
        *self as u16
    }
}




struct Engine {
    lines: Vec<String>, // lines of source code (.txt)
    registers: [Register; 8], // A-D, ESI, EDI, P
    memory_manager: MemoryManager, // 16 KB bytes of memory
    mode: bool, // false = reading data, true = reading code
    stack_pointer: usize, // pointer to the top of the stack within memory,
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
            self.value = (self.value & 0x00FF) | ((value as u16) << 8);
        } else {
            self.value = (self.value & 0xFF00u16 as u16) | (value as u16 & 0x00FF);
        }
    }

    fn load_word(&mut self, value: u16) {
        self.value = value;
    }
}

fn get_register_size(reg_name: &str) -> Option<usize> {
    if reg_name.ends_with('X') || reg_name.ends_with('I') {
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
#[derive(Clone, Copy, Debug)]
struct VariableMetadata {
    start_index: usize,
    length: usize,
    is_array: bool
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
    fn save_variable(&mut self, variable_name: String, data: &[u8], is_array: bool, stack_pointer: usize) -> Result<(), String> {
        let length = data.len();
        if !is_array && length > 2 {
            panic!("Variable too big, must be array.");
        }
        if let Ok(location) = self.find_free_block(length, stack_pointer) {
            // Copy data to the found location
            for (i, &byte) in data.iter().enumerate() {
                self.memory[location + i] = byte;
            }
            println!("[SAVED] Saved variable {} @ {}\n", variable_name, location);

            // Save the metadata with the correct start_index
            self.variable_pointers.insert(variable_name, VariableMetadata {
                start_index: location,
                length,
                is_array,
            });
            Ok(())
        } else {
            Err("Not enough contiguous free memory".to_string())
        }
    }

    fn find_free_block(&mut self, length: usize, stack_pointer: usize) -> Result<usize, ErrorCode> {
        let mut start_index = 0;

        // Iterate over the variable pointers hashmap
        if self.variable_pointers.len() == 0 {
            return Ok(0);
        }

        for (_, metadata) in &self.variable_pointers {
            // Check if there's enough contiguous free memory between allocated blocks
            let end_index = metadata.start_index + metadata.length;

            // Check if there's enough free space between end of previous block and start of current block
            if start_index + length <= metadata.start_index {
                return Ok(start_index);
            }

            start_index = end_index;  // Move start_index to end of current block
            self.check_memory_address(start_index);

        }

        if start_index + length < stack_pointer  {
            return Ok(start_index);
        }
        
        Err(ErrorCode::NotEnoughSpace("Not enough contiguous free memory".to_string()))
    }
    

    fn is_valid_array(&self, text: &str) -> Option<Vec<u8>> {
    // Adjust the regex pattern to correctly capture hexadecimal, binary, and decimal numbers
    let variable_pattern = Regex::new(r"^\[((?:&0x[0-9a-fA-F]+|&0b[01]+|\d+)(?:,(?:&0x[0-9a-fA-F]+|&0b[01]+|\d+))+)\]$").unwrap();
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

    fn is_memory_operand(&self, operand: &str) -> bool {
        operand.starts_with('[') && operand.ends_with(']')
    }

    fn calculate_effective_address(&self, mem_operand: &str, registers: &[Register; 8]) -> Result<usize, ErrorCode> {
        // Remove the square brackets
        if !self.is_memory_operand(mem_operand){
            return Err(ErrorCode::InvalidPointer("Memory Operand must be enveloped in []".to_string()));
        }
        let addr_expression = &mem_operand[1..mem_operand.len() - 1];
        
        let mut effective_address = 0;
        
        for part in addr_expression.split('+') {
            let part = part.trim();
            
            if part.contains('*') {
                let mut iter = part.split('*');
                let scale: usize = iter.next().unwrap().parse().unwrap();
                let reg = iter.next().unwrap();
                effective_address += scale * registers[get_register(reg)].value as usize;
            } else if part.chars().all(char::is_numeric) {
                // Handle displacement (e.g., "4")
                effective_address += part.parse::<isize>().unwrap() as usize;
            } else if let Some(var_metadata) = self.variable_pointers.get(part) {
                // Handle variable name as a direct value // deprecated
                effective_address += var_metadata.start_index;
                // match var_metadata.length {
                //     1 => effective_address += self.memory[var_metadata.start_index] as usize,
                //     2 => {
                //         // Combine two bytes into a 16-bit value
                //         let high_byte = self.memory[var_metadata.start_index] as usize;
                //         let low_byte = self.memory[var_metadata.start_index + 1] as usize;
                //         effective_address += (high_byte << 8) | low_byte;
                //     },
                //     _ => {
                //         // return Err(ErrorCode::InvalidPointer("Variable in effective address cannot be more than 2 bytes long.".to_string()))
                //         effective_address += var_metadata.start_index;
                //     },
                // }

                // Handle variable name as a pointer

            } else {
                // Handle base register (e.g., "EBX")
                effective_address += registers[get_register(part)].value as usize;
            }
        }

        Ok(effective_address as usize)
    }


    // fn is_valid_pointer(&self, text: &str) -> Option<usize> {

    //     // Define the regex patterns
    //     let hex_pattern = Regex::new(r"^&0x([0-9a-fA-F]+)$").unwrap();
    //     let bin_pattern = Regex::new(r"^&0b([01]+)$").unwrap();
    //     let variable_pattern = Regex::new(r"^&([a-zA-Z_]+)$").unwrap();

    //     // Check if the input text matches the hex pattern
    //     if let Some(captures) = hex_pattern.captures(text) {
    //         if let Some(hex_str) = captures.get(1) {
    //             if let Ok(address) = usize::from_str_radix(hex_str.as_str(), 16) {
    //                 // Check if the address is within the valid range (16K)
    //                 if address < 1024 * 16 {
    //                     return Some(address)
    //                 } else {
    //                     return None
    //                 }
    //             }
    //         }
    //     }
    
    //     // Check if the input text matches the binary pattern
    //     if let Some(captures) = bin_pattern.captures(text) {
    //         if let Some(bin_str) = captures.get(1) {
    //             if let Ok(address) = usize::from_str_radix(bin_str.as_str(), 2) {
    //                 // Check if the address is within the valid range (16K)
    //                 if address < 1024 * 16 {
    //                     return Some(address)
    //                 } else {
    //                     return None
    //                 }
    //             }
    //         }
    //     }


    //     if let Some(captures) = variable_pattern.captures(text) {
    //         if let Some(variable_name) = captures.get(1) {
    //             if let Some(address) = self.variable_pointers.get(variable_name.as_str()) {
    //                 // Check if the address is within the valid range (16K)
    //                 if address.start_index < 1024 * 16 {
    //                     return Some(address.start_index)
    //                 } else {
    //                     return None
    //                 }
    //             }
    //         }
    //     }
    
    //     // If it doesn't match any pattern, return false
    //     None
    // }

    // fn get_variable_data(&self, variable_name: &str) -> Option<Vec<u8>> {
    //     if let Some(metadata) = self.variable_pointers.get(variable_name) {
    //         let start_index = metadata.start_index;
    //         Some(self._fetch_memory(start_index, metadata.length))
    //     }else {
    //         None
    //     }
    // }

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

fn get_register(name: &str) -> usize{
    match name {
        // "AX" => &mut self.registers[0],
        // "BX" => &mut self.registers[1],
        // "CX" => &mut self.registers[2],
        // "DX" => &mut self.registers[3],
        // "ESI" => &mut self.registers[4],
        // "EDI" => &mut self.registers[5],
        // "IP" => &mut self.registers[6],   
        // "FLAG" => &mut self.registers[7],   
        // _ => panic!("Invalid register name"),

        "AX"|"AL"|"AH" => 0,
        "BX"|"BL"|"BH" => 1,
        "CX"|"CL"|"CH" => 2,
        "DX"|"DL"|"DH" => 3,
        "ESI" => 4,
        "EDI" => 5,
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

    fn get_variable_metadata(&self, text: &str) -> Option<VariableMetadata> {
        self.memory_manager.variable_pointers.get(text).cloned()
    }


    fn addition_mem(&mut self, dest: usize, value: u16, word: bool) {
        self.memory_manager.check_memory_address(dest);
        if word {
            self.memory_manager.check_memory_address(dest+1);

            let mem_top_byte = (self.memory_manager.memory[dest] as u16) << 8;
            let mem_bottom_byte = self.memory_manager.memory[dest] as u16;

            let word_value = mem_top_byte | mem_bottom_byte;

            let (sum, overflowed) = word_value.overflowing_add(value);

            self.memory_manager.memory[dest] = (sum >> 8) as u8;
            self.memory_manager.memory[dest+1] = (sum & 0x0FF) as u8;

            self.set_flags(sum, overflowed);

        } else {
            
            if value > 127 {
                panic!("Single byte value can't be over 127");
            }

            let byte_value = self.memory_manager.memory[dest];

            let (sum, overflowed) = byte_value.overflowing_add(value as u8);

            self.memory_manager.memory[dest] = sum;

            self.set_flags(sum as u16, overflowed);
        }
    }

    fn addition_reg(&mut self, dest: &str, value_to_add: u16) {

        let dest_register = get_register(dest);

        match dest.chars().last() {
            Some('L') | Some('H') => {
                let is_top = dest.chars().last() == Some('H');
                let current_value = self.registers[dest_register].get_byte(is_top);

                if value_to_add > u8::MAX as i16 as u16 || value_to_add < u8::MIN as i16 as u16 {
                    panic!("Value cannot fit in destination");
                }
                let (result, overflowed) = current_value.overflowing_add(value_to_add as u8);

                self.registers[dest_register].load_byte(result as u8, is_top);

                self.set_flags(result as u16, overflowed);
            },
            Some('X') | Some('I') => {
                let current_value = self.registers[dest_register].value;
                let (result, overflowed) = current_value.overflowing_add(value_to_add);
                self.registers[dest_register].load_word(result); // Load low byte into AH
                self.set_flags(result, overflowed);
            },
            _ => panic!("Invalid destination"),
        };
    }

    fn get_register_value(&self, name: &str) -> u16 {
        let value = self.registers[get_register(name)].value;

        match name {
            "AL" | "BL" | "CL" | "DL" => value & 0x00FF,
            "AH" | "BH" | "CH" | "DH" => value >> 8,
            _ => value,
        }
    }

    fn set_flags(&mut self, result: u16, overflowed: bool) {
        self.registers[get_register("FLAG")].value = 0; // Reset all flags

        // Set Carry Flag
        // if carry {
        //     self.registers[get_register("FLAG")].value |= Flag::Carry.value();
        // }

        // Set Parity Flag
        if result.count_ones() % 2 == 0 {
            self.registers[get_register("FLAG")].value |= Flag::Parity.value();
        }

        // Set Auxiliary Carry Flag
        // if auxiliary_carry {
        //     self.registers[get_register("FLAG")].value |= Flag::AuxiliaryCarry.value();
        // }

        // Set Zero Flag
        if result == 0 {
            self.registers[get_register("FLAG")].value |= Flag::Zero.value();
        }

        // Set Sign Flag
        if result >> 15 == 1 {
            self.registers[get_register("FLAG")].value |= Flag::Sign.value();
        }

        // Set Overflow Flag
        if overflowed {
            self.registers[get_register("FLAG")].value |= Flag::Overflow.value();
        }
        
    }
    fn execute(&mut self) {
        let re = Regex::new(r#", | "#).unwrap();
        let mut ip = self.registers[6].value as usize;

        while ip < self.lines.len() {
            // Split the string and collect into a vector of &str
            ip = self.registers[6].value as usize;
            let whole_line = self.lines[ip].clone();
            let line_vec: Vec<&str> = whole_line.split("#").collect();
            let arguments: Vec<&str> = re.split(&line_vec[0])
                                        .filter(|&s| !s.is_empty())
                                        .collect();
            
            println!("[LINE]  [{ip}]\t{whole_line}");

            // Process the instruction based on the arguments
            match arguments.as_slice() {
                ["mov", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                    self.mov_reg_reg(reg_dst, reg_src);
                },
                
                // Variables and mem_addresses are basically the same so let's sort this out.
                // let i, 5 # not an array.
                // mov AX, i # This will mov the address of i into AX
                // mov AX, [i] # This will mov the value of i into AX

                // let arr, [1,2,3] # array
                // mov AX, arr # This will move the address of the beginning of the array into AX
                // mov AX, [arr] # This will move the first byte of the array into AX
                
                ["mov", reg, variable] if self.is_valid_register(*reg) && self.get_variable_metadata(*variable).is_some() => {
                    if let Some(metadata) = self.get_variable_metadata(*variable) {
                        self.mov_reg_const(*reg, metadata.start_index as u16);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },

                ["mov", reg, constant] if self.is_valid_register(*reg) && parse_value_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        self.mov_reg_const(reg, value as u16);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },

                ["mov", reg, mem_address] if self.is_valid_register(*reg) && self.memory_manager.is_memory_operand(mem_address) => {
                   
                    match self.memory_manager.calculate_effective_address(*mem_address, &self.registers){
                        Ok(parsed_address) => self.mov_reg_mem(reg, parsed_address),
                        Err(err_code) => {
                            match err_code {
                                ErrorCode::InvalidPointer(err_message) => println!("Invalid pointer error: {}", err_message),
                                _ => {},
                            }
                        }
                    }

                },

                ["mov", mem_address, reg] if self.is_valid_register(*reg) && self.memory_manager.is_memory_operand(mem_address) => {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                        self.mov_mem_reg(parsed_address, reg);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },
                
                ["mov", mem_address, constant] if parse_value_to_usize(*constant).is_some() && self.memory_manager.is_memory_operand(*mem_address) => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(*mem_address, &self.registers) {
                            self.memory_manager.memory[parsed_address] = value as u8;
                            self.set_flags(value as u16, false);
                        } else {
                             println!("[ERROR] [{ip}] Something went wrong.");
                        }
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },

                ["mov", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Mov));
                },

                // ADD Instructions
                ["add", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                    self.add_reg_reg(reg_dst, reg_src);
                },
                ["add", reg_dst, mem_address] if self.memory_manager.is_memory_operand(mem_address) => {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                        self.add_reg_mem(reg_dst, parsed_address);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },
                ["add", mem_address, reg_src] if self.memory_manager.is_memory_operand(mem_address) && self.is_valid_register(reg_src)=> {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                        self.add_mem_reg(parsed_address, reg_src);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },
                ["add", reg, constant] if self.is_valid_register(*reg) && parse_value_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        self.add_reg_const(reg, value as u16);
                    } else {

                    }
                },
                // Variables and mem_addresses are basically the same so let's sort this out.
                // let i, 5 # not an array.
                // add AX, i  # Adds the address of i to AX
                // add AX, [i]  # Adds the value of i to AX

                // let arr, [1,2,3] # array
                // mov AX, arr # This will move the address of the beginning of the array into AX
                // mov AX, [arr] # This will move the first byte of the array into AX
                ["add", reg, variable] if self.is_valid_register(*reg) && self.get_variable_metadata(*variable).is_some() => {
                    if let Some(metadata) = self.get_variable_metadata(*variable) {
                        self.add_reg_const(*reg, metadata.start_index as u16);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },

                ["add", mem_address, constant] if parse_value_to_usize(*constant).is_some() && self.memory_manager.is_memory_operand(*mem_address) => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(*mem_address, &self.registers) {
                            let (result, overflowed) = self.memory_manager.memory[parsed_address].overflowing_add(value as u8);
                            self.set_flags(value as u16, overflowed);
                            self.memory_manager.memory[parsed_address] = result;
                        } else {
                             println!("[ERROR] [{ip}] Something went wrong.");
                        }
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },



                ["add", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Add));
                },

                // SUB Instructions
                ["sub", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                    self.sub_reg_reg(reg_dst, reg_src);
                },

                ["sub", reg_dst, mem_address] if self.memory_manager.is_memory_operand(mem_address) => {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                        self.sub_reg_mem(reg_dst, parsed_address);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },
                ["sub", mem_address, reg_src] if self.memory_manager.is_memory_operand(mem_address) && self.is_valid_register(reg_src)=> {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                        self.sub_mem_reg(parsed_address, reg_src);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },
                ["sub", reg, constant] if self.is_valid_register(*reg) && parse_value_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        self.sub_reg_const(reg, value as u16);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },
                ["sub", reg, variable] if self.is_valid_register(*reg) && self.get_variable_metadata(*variable).is_some() => {
                    if let Some(metadata) = self.get_variable_metadata(*variable) {
                        self.add_reg_const(*reg, metadata.start_index as u16);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },

                ["sub", reg, variable] if self.is_valid_register(*reg) && self.get_variable_metadata(*variable).is_some() => {
                    if let Some(metadata) = self.get_variable_metadata(*variable) {
                        self.sub_reg_const(*reg, metadata.start_index as u16);
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },

                ["sub", mem_address, constant] if parse_value_to_usize(*constant).is_some() && self.memory_manager.is_memory_operand(*mem_address) => {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(*mem_address, &self.registers) {
                            let (result, overflowed) = self.memory_manager.memory[parsed_address].overflowing_sub(value as u8);
                            self.set_flags(value as u16, overflowed);
                            self.memory_manager.memory[parsed_address] = result;
                        } else {
                             println!("[ERROR] [{ip}] Something went wrong.");
                        }
                    } else {
                         println!("[ERROR] [{ip}] Something went wrong.");
                    }
                },

                ["sub", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Sub));
                },
                // // MUL Instructions
                // ["mul", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                //     self.mul_reg_reg(reg_dst, reg_src);
                // },
                // ["mul", reg_dst, mem_address] if self.memory_manager.is_memory_operand(mem_address) => {
                //     if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                //         self.mul_reg_mem(reg_dst, parsed_address);
                //     }
                // },
                // ["mul", reg, constant] if self.is_valid_register(*reg) && parse_value_to_usize(*constant).is_some() => {
                //     if let Some(value) = parse_value_to_usize(*constant) {
                //         self.mul_reg_const(reg, value as u16);
                //     }
                // },
                // ["mul", reg, variable] if self.is_valid_register(*reg) && self.get_variable(*variable).is_some() => {
                //     if let Some(_) = self.get_variable(*variable) {
                //         self.mul_reg_variable(reg, variable);
                //     }
                // },
                ["mul", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Mul));
                },
                // // DIV Instructions
                // ["div", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                //     self.div_reg_reg(reg_dst, reg_src);
                // },
                // ["div", reg_dst, mem_address] if self.memory_manager.is_memory_operand(mem_address) => {
                //     if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                //         self.div_reg_mem(reg_dst, parsed_address);
                //     }
                // },
                // ["div", mem_address, reg_src] if self.memory_manager.is_memory_operand(mem_address) => {
                //     if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                //         self.div_mem_reg(parsed_address, reg_src);
                //     }
                // },
                // ["div", reg, constant] if self.is_valid_register(*reg) && parse_value_to_usize(*constant).is_some() => {
                //     if let Some(value) = parse_value_to_usize(*constant) {
                //         self.div_reg_const(reg, value as u16);
                //     }
                // },
                // ["div", reg, variable] if self.is_valid_register(*reg) && self.get_variable(*variable).is_some() => {
                //     if let Some(_) = self.get_variable(*variable) {
                //         self.div_reg_variable(reg, variable);
                //     }
                // },
                ["div", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Div));
                },
                ["print", mem_address] if self.memory_manager.is_memory_operand(mem_address) => {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers) {
                        let ip = self.registers[get_register("IP")].value;
                        println!("\n[PRINT] [{ip}]: {0}\n", self.memory_manager.memory[parsed_address]);
                    }
                },
                ["let", variable_name, constant] if self.memory_manager.is_valid_variable_name(variable_name) &&
                (parse_value_to_usize(*constant).is_some() || self.memory_manager.is_valid_array(*constant).is_some() )=> {
                    if let Some(value) = parse_value_to_usize(*constant) {
                        if value > u8::MAX.into() {
                            let low_byte = (value & 0x00FF) as u8;
                            let high_byte = ((value >> 8) & 0x00FF) as u8;
            
                            self.memory_manager.save_variable(variable_name.to_string(), &[high_byte as u8, low_byte as u8], false, self.stack_pointer)
                                .unwrap_or_else(|err| eprintln!("Error saving variable: {}", err));
                        } else {
                            self.memory_manager.save_variable(variable_name.to_string(), &[value as u8], false, self.stack_pointer)
                                .unwrap_or_else(|err| eprintln!("Error saving variable: {}", err));
                        }
                    } else if let Some(array) = self.memory_manager.is_valid_array(*constant) {
                        self.memory_manager.save_variable(variable_name.to_string(), &array, true, self.stack_pointer)
                            .unwrap_or_else(|err| eprintln!("Error saving variable: {}", err));
                    }
                },
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
        print!("[");
        for i in 0..10 {
            print!("{}", self.memory_manager.memory[i]);
            if i < 9 {
                print!(", ")
            }
        }
        println!("]");
    }

    fn check_register_sizes(&self, dest: &str, src: &str) {
        let size_dest = get_register_size(dest);
        let size_src = get_register_size(src);

        if size_src != size_dest {
            panic!("Invalid Size");
        }
    }

    // fn check_register_size_var(&self, dest: &str, length: usize) {
    //     let size = match length {
    //         1 => 'L',
    //         2 => 'X',
    //         _ => 'E',
    //     };
    
    //     if (dest.ends_with('L') || dest.ends_with('H')) && size == 'X' {
    //         panic!("Invalid Size!"); // Throw an error if size 'X' (16-bit) is attempted with 'L' or 'H'
    //     }
    // }
    



    // MOV operations
    // REG <- Reg, Const, Var, Mem
    // Mem <- Reg, Const, Var, Mem
    // Var <- Reg, Const, Var, Mem
    // Const <- NOTHING, Const can't be moved into

    fn mov_reg_reg(&mut self, dest: &str, src: &str) {
        self.check_register_sizes(dest, src);

        let result = match (src.chars().last(), dest.chars().last()) {
            (Some('L')|Some('H'), Some('L')|Some('H')) => {

                let src_register_value = self.get_register_value(src) as u8;

                let top_dst: bool = dest.chars().last() == Some('H');
                self.registers[get_register(dest)].load_byte(src_register_value, top_dst);
                src_register_value as u16

            },
            (Some('X') | Some('I'), Some('X') | Some('I')) => {
                let src_register_value = self.get_register_value(src);
                self.registers[get_register(dest)].load_word(src_register_value);
                src_register_value
            }
            _ => {0}
        };
        self.set_flags(result, false);
    }
    fn mov_reg_const(&mut self, dest: &str, constant: u16) {
        match dest.chars().last() {
            Some('L') => self.registers[get_register(dest)].load_byte(constant as u8, false),
            Some('H') => self.registers[get_register(dest)].load_byte(constant as u8, true),
            Some('X') | Some('I') => self.registers[get_register(dest)].load_word(constant),
            _ => println!("[ERROR] Invalid register")
        }
        self.set_flags(constant, false);
    }

    fn add_reg_const(&mut self, dest: &str, constant: u16) {
        self.addition_reg(dest, constant);
    }
    

    fn mov_reg_mem(&mut self, dest: &str, mem_address: usize) {
        let result: u16 = match dest.chars().last() {
            Some('L')|Some('H') => {
                let is_top = dest.chars().last() == Some('H');
                self.registers[get_register(dest)].load_byte(self.memory_manager.memory[mem_address], is_top);
                self.memory_manager.memory[mem_address] as u16
            },
            Some('X') | Some('I') => {
                self.memory_manager.check_memory_address(mem_address + 1);
                let result = ((self.memory_manager.memory[mem_address] as u16) << 8) + self.memory_manager.memory[mem_address + 1] as u16;
                self.registers[get_register(dest)].load_word(result);
                result
            },
            _ => {0}
        };
        self.set_flags(result, false);

    }

    fn mov_mem_reg(&mut self, mem_address: usize, src: &str) {
        match src.chars().last() {
            Some('L')|Some('H') => {
                let value = self.get_register_value(src);
                self.memory_manager.memory[mem_address] = value as u8;
                self.set_flags(value, false);                
            },
            Some('X') | Some('I') => {
                self.memory_manager.check_memory_address(mem_address + 1);
                self.memory_manager.memory[mem_address]     = self.registers[get_register(src)].get_byte(true);
                self.memory_manager.memory[mem_address + 1] = self.registers[get_register(src)].get_byte(false);
                self.set_flags(self.registers[get_register(src)].get_word(), false);
            },
            _ => println!("[ERROR] Invalid register")
        }
    }

    fn add_reg_reg(&mut self, dest: &str, src: &str) {
        self.check_register_sizes(dest, src);
        let dest_register = get_register(dest);
        let src_register = get_register(src);

        let (result, overflowed): (u16, bool) = {
            match dest.chars().last() {
                Some('X') | Some('I') => {
                    let dest_value = self.registers[dest_register].get_word();
                    let src_value = self.registers[src_register].get_word();

                    let (sum, overflowed) = dest_value.overflowing_add(src_value);
                    (sum, overflowed)
                },
                Some('L') | Some('H') => {
                    let is_top = dest.chars().last() == Some('H');
                    let dest_value = self.registers[dest_register].get_byte(is_top);
                    let src_value = self.registers[src_register].get_byte(is_top);

                    let (sum, overflowed) = dest_value.overflowing_add(src_value);
                    (sum as u16, overflowed)
                },
                _ => {(0, false)},
            }
        };

        self.set_flags(result, overflowed);
        self.mov_reg_const(dest, result);

    }

    fn add_mem_reg(&mut self, mem_address: usize, src: &str) {
        match src.chars().last() {
            Some('L') | Some('H') => {
                let is_top = src.chars().last() == Some('H');
                let value = self.registers[get_register(src)].get_byte(is_top);
                self.addition_mem(mem_address, value as u16, false);
            },
            Some('X') | Some('I') => {
                self.memory_manager.check_memory_address(mem_address + 1);
                let value = self.registers[get_register(src)].get_word();
                self.addition_mem(mem_address, value, true);
            },
            _ => println!("[ERROR] Invalid register")
        }
    }
    
    fn add_reg_mem(&mut self, dest: &str, mem_address: usize) {
        self.memory_manager.check_memory_address(mem_address);
    
        let (result, overflowed) = {
            match dest.chars().last() {
                Some('X') | Some('I') => {
                    let dest_value = self.registers[get_register(dest)].get_word();
                    let word_bytes = (self.memory_manager.memory[mem_address], self.memory_manager.memory[mem_address + 1]);
                    let mem_value = ((word_bytes.0 as u16) << 8) | (word_bytes.1 as u16);
                    dest_value.overflowing_add(mem_value)
                },
                Some('L') | Some('H') => {
                    let is_top = dest.chars().last() == Some('H');
                    let dest_value = self.registers[get_register(dest)].get_byte(is_top);
                    let mem_value = self.memory_manager.memory[mem_address];
                    let (sum, overflowed) = dest_value.overflowing_add(mem_value);
                    (sum as u16, overflowed)
                },
                _ => (0, false),
            }
        };
    
        self.set_flags(result, overflowed);
        self.mov_reg_const(dest, result);
    }

    
    //////////// SUB /////////////
    fn sub_reg_reg(&mut self, dest: &str, src: &str) {
        self.check_register_sizes(dest, src);

        match dest.chars().last() {
            Some('X') | Some('I') => {
                let value_src = self.registers[get_register(src)].get_word();
                let value_dst=  self.registers[get_register(dest)].get_word();
                let (result, overflowed) = value_src.overflowing_sub(value_dst);
                self.registers[get_register(dest)].load_word(result);
                self.set_flags(result as u16, overflowed);

            },
            Some('L')|Some('H') => {
                let is_top_src = src.chars().last() == Some('H');
                let is_top_dst = dest.chars().last() == Some('H');

                let value_src = self.registers[get_register(src)].get_byte(is_top_src);
                let value_dst=  self.registers[get_register(dest)].get_byte(is_top_dst);
                
                let (result, overflowed) = value_src.overflowing_sub(value_dst);
                self.registers[get_register(dest)].load_byte(result, is_top_dst);
                self.set_flags(result as u16, overflowed);
            }
            _ => println!("[ERROR] Invalid register")
        }

    }

    fn sub_mem_reg(&mut self, mem_address: usize, src: &str) {
        match src.chars().last() {
            Some('L') | Some('H') => {
                let is_top = src.chars().last() == Some('H');
                let value = self.registers[get_register(src)].get_byte(is_top);
                self.addition_mem(mem_address, !value as u16, false);
            },
            Some('X') | Some('I') => {
                self.memory_manager.check_memory_address(mem_address + 1);
                let value = self.registers[get_register(src)].get_word();
                self.addition_mem(mem_address, !value, true);
            },
            _ => {println!("[ERROR] Invalid register");}
        }
    }
    
    fn sub_reg_mem(&mut self, dest: &str, mem_address: usize) {
        self.memory_manager.check_memory_address(mem_address);
        self.memory_manager.memory[mem_address] = !self.memory_manager.memory[mem_address]+1; // invert
        self.add_reg_mem(dest, mem_address);
        self.memory_manager.memory[mem_address] = !self.memory_manager.memory[mem_address]+1; // bring it back
    }
    
    fn sub_reg_const(&mut self, dest: &str, constant: u16) {
        let dest_value = self.registers[get_register(dest)].value;
        let (result, overflowed) = match dest.chars().last() {
            Some('X') | Some('I') => {
                let (value, overflowed) = self.registers[get_register(dest)].value.overflowing_sub(dest_value.wrapping_sub(constant));
                self.registers[get_register(dest)].load_word(value);
                (value, overflowed)
            }
            Some('L')|Some('H') => {
                let top = dest.chars().last()==Some('H');
                let dest_value_l = self.registers[get_register(dest)].get_byte(top);
                let (value, overflowed) = dest_value_l.overflowing_sub(constant as u8);
                self.registers[get_register(dest)].load_byte(value, top);
                (value as u16, overflowed)
            },
            _ => {(0, false)},
        };
        self.set_flags(result, overflowed);

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


    ////////// MUL ///////////

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
    // for line in &assembly.lines {
    //     println!("{}", line);
    // }
    assembly.execute();

    // Optionally, print out the registers to verify
    for register in &assembly.registers {
        let low_byte = (register.value & 0xFF) as u8;
        let high_byte = ((register.value >> 8) & 0xFF) as u8;
        println!("Register {}:\t{}\t({:08b} {:08b})", register.name, register.value, high_byte, low_byte);
    }
    Ok(())
}

