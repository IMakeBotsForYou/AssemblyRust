use crate::{
    error_code::ErrorCode,
    flag::Flag,
    instruction::Instruction,
    line_processor::LineProcessor,
    memory_manager::MemoryManager,
    register::{get_register_size, Register, RegisterName},
    // status::Status,
    utils::{parse_string_to_usize, read_lines_from_file},
    variable_metadata::{
        // VariableMetadata,
        VariableSize,
    },
};
use std::fmt;
use std::io::{self, stdin, Write};

const MEMORY_SIZE: usize = 1024 * 16; // 16 KB

fn skip_lines(lines_to_skip: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    // Move cursor up by `lines_to_skip` lines without clearing them
    for _ in 0..lines_to_skip {
        write!(stdout, "\x1B[B")?; // ANSI code to move cursor up one line
    }

    // Flush output to ensure everything is printed
    stdout.flush()?;

    Ok(())
}

fn clear_screen(rows: usize) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Move the cursor to the beginning of the terminal
    write!(stdout, "\x1B[H")?;

    // Clear the first N rows
    for row in 0..rows {
        // Move the cursor to the beginning of the current row
        write!(stdout, "\x1B[{};0H", row + 1)?;
        // Clear the current line
        write!(stdout, "\x1B[2K")?;
    }
    write!(stdout, "\x1B[H")?; // Return to the beginning of the terminal
                               // Make sure all commands are executed
    stdout.flush()?;

    Ok(())
}

fn pause() {
    let mut s = String::new();
    let _ = stdin().read_line(&mut s);
}

fn back_to_str(vec: &[String]) -> String {
    let mut ret = String::new();
    
    ret.push_str(&format!("{} ", vec[0]));

    for item in vec.iter().skip(1) {
        ret.push_str(&format!("{}, ", item));
    }
    let f = if vec.len() == 1 { 1 } else { 2 };
    ret[..ret.len() - f].to_string()
}

fn combine_parts(vec: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut combined: String = String::new();
    let mut in_quotes: bool = false;

    for item in vec {
        if item.starts_with('\'') && item.ends_with('\'') {
            result.push(format!("\'{}\'", item.trim_matches('\'')));
        } else if item.starts_with('\'') {
            combined.push_str(&format!("\'{}", item.trim_matches('\'')));
            in_quotes = true;
        } else if item.ends_with('\'') {
            combined.push_str(", ");
            combined.push_str(&format!("{}\'", item.trim_matches('\'')));
            result.push(combined.clone());
            combined.clear();
            in_quotes = false;
        } else if in_quotes {
            combined.push_str(", ");
            combined.push_str(item);
        } else {
            result.push(item.to_string());
        }
    }

    // If there's a trailing combined string without a closing quote
    if !combined.is_empty() {
        result.push(combined);
    }

    result
}
#[allow(unused_assignments)]
pub struct Engine {
    pub lines: LineProcessor,      // lines of source code (.txt)
    pub registers: [Register; 10], // A-D, ESI, EDI, P
    memory_manager: MemoryManager, // 16 KB bytes of memory
                                   // mode: bool, // false = reading data, true = reading code
                                   // status: Status, // status: ok, error, halted,
                                   // interrupts: Vec<Interrupt>
}

impl Engine {
    pub fn new(file_name: &str) -> io::Result<Self> {
        let file_lines = read_lines_from_file(file_name)?;
        let my_registers: [Register; 10] = [
            Register::new(RegisterName::EAX),
            Register::new(RegisterName::EBX),
            Register::new(RegisterName::ECX),
            Register::new(RegisterName::EDX),
            Register::new(RegisterName::ESI),
            Register::new(RegisterName::EDI),
            Register::new(RegisterName::BP),
            Register::new(RegisterName::SP),
            Register::new(RegisterName::IP),
            Register::new(RegisterName::FLAG),
        ];

        let ds = 0_usize; // DATA SEGMENT starts at 0
        let cs = 1024 * 3_usize; // CODE SEGMENT starts at 3072
        let ss: usize = MEMORY_SIZE - 1024; // STACK SEGMENT, starts at 15360 (1024*15)
        Ok(Self {
            lines: LineProcessor::new(file_lines),
            registers: my_registers,
            memory_manager: MemoryManager::new(MEMORY_SIZE, [ds, cs, ss]),
        })
    }
    // Only used for tests
    pub fn get_memory(&self, amount: usize) -> Vec<u8> {
        self.memory_manager._get_memory(0, amount)
    }

    pub fn get_register_value(&self, reg_name: &RegisterName) -> u32 {
        let reg = &self.registers[reg_name.to_index()];
        let value = reg.get_dword();
        match get_register_size(reg_name) {
            VariableSize::Byte => {
                if reg_name.is_top().unwrap() {
                    (value & 0x0000FF00) >> 8
                } else {
                    value & 0x000000FF
                }
            }
            VariableSize::Word => value & 0x0000FFFF,
            VariableSize::DoubleWord => value,
        }
    }

    fn set_flags(&mut self, result: usize, size: VariableSize, overflowed: bool) {
        let _ = self.set_register_value(&RegisterName::FLAG, 0);

        if result.count_ones() % 2 == 0 {
            self.set_flag(Flag::Parity, true);
        }

        // Set Zero Flag
        if result == 0 {
            self.set_flag(Flag::Zero, true);
        }

        // Set Sign Flag
        let mask = 0xFF_usize << ((size.value() - 1) * 8);
        let displacement = size.value() * 8 - 1;
        if (result & mask) >> displacement == 1 {
            self.set_flag(Flag::Sign, true);
        }

        // Set Overflow Flag
        if overflowed {
            self.set_flag(Flag::Overflow, true);
        }
    }

    // Also trims the parameter if it's a pointer (removes BYTE/WORD/DWORD PTR)
    fn get_argument_size<'a>(&self, argument: &'a str) -> (Option<VariableSize>, &'a str) {
        let parts: Vec<&str> = argument.split_whitespace().collect();
        match parts.as_slice() {
            ["BYTE", "PTR", arg] => (Some(VariableSize::Byte), *arg),
            ["WORD", "PTR", arg] => (Some(VariableSize::Word), *arg),
            ["DWORD", "PTR", arg] => (Some(VariableSize::DoubleWord), *arg),
            [arg] => {
                if let Ok((_, size)) = self.parse_value_from_parameter(arg, None) {
                    (Some(size), *arg)
                } else {
                    (None, *arg)
                }
            }
            _ => (None, argument),
        }
    }

    pub fn parse_value_from_parameter(
        &self,
        parameter: &str,
        mut size: Option<VariableSize>,
    ) -> Result<(u32, VariableSize), ErrorCode> {
        if let Some(value) = parse_string_to_usize(parameter) {
            // Handle constant values
            let vi32 = value as i32;
            let assumed_size = if vi32 >= i8::MIN.into() && vi32 <= u8::MAX.into() {
                VariableSize::Byte
            } else if vi32 >= i16::MIN.into() && vi32 <= u16::MAX.into() {
                VariableSize::Word
            } else {
                VariableSize::DoubleWord
            };
            // else {
            //     return Err(ErrorCode::InvalidValue(format!(
            //         "Value {} is outside of allowed range. 0-u32::MAX",
            //         value
            //     )));
            // };
            return Ok((value, assumed_size));
        }

        if let Ok(reg_name) = RegisterName::from_str_to_reg_name(parameter) {
            // Handle register values
            let assumed_size = get_register_size(&reg_name);
            let value = self.get_register_value(&reg_name);
            return Ok((value, assumed_size));
        }

        if let Ok(parsed_address) =
            self.memory_manager
                .calculate_effective_address(parameter, &self.registers, true)
        {
            // Handle memory addresses
            let memory_address_str = parameter.to_string();
            if memory_address_str.len() < 3 {
                return Err(ErrorCode::InvalidPointer(
                    "Pointer length must be more than 3 including []".to_string(),
                ));
            }
            // No assumed size. If the parameter is a variable [var], then get var's size at declaration
            // Example: var dw 5
            // mov eax, [var] ; [var] is assumed as a word pointer

            if size.is_none() {
                if let Some(metadata) = self
                    .memory_manager
                    .get_variable(&memory_address_str[1..memory_address_str.len() - 1])
                {
                    size = Some(metadata.size);
                } else if self
                    .memory_manager
                    .labels
                    .contains_key(&memory_address_str[1..memory_address_str.len() - 1])
                {
                    size = Some(VariableSize::Word);
                }
            }
            // If we still have no assumed size, the pointer defaults to a byte pointer.
            match size.unwrap_or(VariableSize::Byte) {
                VariableSize::Byte => {
                    let a = self.memory_manager.get_byte(parsed_address)?;
                    Ok((a as u32, VariableSize::Byte))
                }
                VariableSize::Word => {
                    let a = self.memory_manager.get_word(parsed_address)?;
                    Ok((a as u32, VariableSize::Word))
                }
                VariableSize::DoubleWord => {
                    let a = self.memory_manager.get_dword(parsed_address)?;
                    Ok((a, VariableSize::DoubleWord))
                }
            }
        } else {
            Err(ErrorCode::InvalidValue(format!(
                "Parameter {} could not be parsed.",
                parameter
            )))
        }
    }
    pub fn is_valid_register(name: &str) -> bool {
        RegisterName::from_str_to_reg_name(name).is_ok()
    }
    pub fn execute(&mut self, debug: bool) -> Result<(), ErrorCode> {
        // Go over code to get all the labels.
        let mut in_proc = false;
        let mut current_proc_start_ip: usize = 0;
        let mut current_proc_name = String::new();
        let mut lines_to_skip = 1;
        let ip_index = RegisterName::IP.to_index();
        loop {
            let line_option = self.lines.next_line();
            if line_option.is_none() {
                break;
            }
            if let Some(line) = line_option {
                let proc_string = "PROC".to_string();
                let end_proc_string = "END".to_string();
                match line.as_slice() {
                    [label]
                        if label.ends_with(':')
                            && self
                                .memory_manager
                                .is_valid_variable_name(&label[..label.len() - 1]) =>
                    {
                        let no_colon = label[..label.len() - 1].to_string();
                        if self.memory_manager.labels.contains_key(&no_colon) {
                            return Err(ErrorCode::LabelAlreadyExists(format!(
                                "Label {no_colon} already exists"
                            )));
                        }
                        self.memory_manager.save_label(
                            no_colon,
                            self.get_register_value(&RegisterName::IP) as usize,
                        )?
                    }
                    [proc_name, proc] if proc == &proc_string => {
                        if self.memory_manager.procs.contains_key(proc_name) {
                            return Err(ErrorCode::LabelAlreadyExists(format!(
                                "Proc {proc_name} already exists"
                            )));
                        }
                        if in_proc {
                            return Err(ErrorCode::InvalidOpcode("Cannot start new proc while in another.".to_string()));
                        }
                        in_proc = true;
                        current_proc_name.clone_from(proc_name);
                        current_proc_start_ip = self.get_register_value(&RegisterName::IP) as usize;
                    }

                    [proc_name, proc] if proc == &end_proc_string => {
                        if self.memory_manager.procs.contains_key(proc_name) {
                            return Err(ErrorCode::LabelAlreadyExists(format!(
                                "Proc {proc_name} already exists"
                            )));
                        }
                        if !in_proc {
                            return Err(ErrorCode::InvalidOpcode("Cannot end proc outside of a proc.".to_string()));
                        }
                        in_proc = false;
                        let current_proc_end_ip =
                            self.get_register_value(&RegisterName::IP) as usize;
                        self.memory_manager.procs.insert(
                            current_proc_name.clone(),
                            (current_proc_start_ip + 1, current_proc_end_ip + 1),
                        );
                    }
                    _ => {}
                }
            }
            self.lines.update_ip_register(&mut self.registers[ip_index]);
        }

        self.lines.set_ip(0);
        self.lines.update_ip_register(&mut self.registers[ip_index]);
        let mut buffer: Option<(usize, String)> = None;
        if debug {
            let _ = clear_screen(100);
        }
        loop {
            let line_option = self.lines.next_line();
            self.lines.update_ip_register(&mut self.registers[ip_index]);
            let ip: usize = self.get_register_value(&RegisterName::IP) as usize;

            if line_option.is_none() {
                break;
            }
            let line = line_option.unwrap();
            if debug {
                let _ = clear_screen(14);
                if let Some((prev_ip, prev_line)) = buffer {
                    println!("[{}]: {}", prev_ip, prev_line); // Previous
                } else {
                    println!();
                }
                println!("[{}]: {} <- YOU ARE HERE", ip, back_to_str(&line)); // Current

                if let Some((next_line, next_ip)) = self.lines.peak() {
                    println!("[{}]: {}", next_ip, back_to_str(&next_line)); // Next
                    buffer = Some((ip, back_to_str(&line)));
                } else {
                    println!();
                    buffer = None;
                }

                println!("{}", self);

                pause();
            }

            let combining_inside_quotes: Vec<String> = combine_parts(&line);
            let line_str: Vec<&str> = combining_inside_quotes.iter().map(|s| &**s).collect();
            match line_str.as_slice() {
                // INC DEC
                [op @ ("inc" | "dec"), register] if RegisterName::is_valid_name(register) => {
                    let register = &RegisterName::from_str_to_reg_name(register).unwrap();
                    let inc = *op == "inc";
                    let (result, overflowed) = match get_register_size(register) {
                        VariableSize::Byte => {
                            let (value, overflowing) = if inc {
                                (self.get_register_value(register) as u8).overflowing_add(1)
                            } else {
                                (self.get_register_value(register) as u8).overflowing_sub(1)
                            };

                            self.set_register_value(register, value as u32)?;
                            (value as u32, overflowing)
                        }
                        VariableSize::Word => {
                            let (value, overflowing) = if inc {
                                (self.get_register_value(register) as u16).overflowing_add(1)
                            } else {
                                (self.get_register_value(register) as u16).overflowing_sub(1)
                            };

                            self.set_register_value(register, value as u32)?;
                            (value as u32, overflowing)
                        }
                        VariableSize::DoubleWord => {
                            let (value, overflowing) = if inc {
                                self.get_register_value(register).overflowing_add(1)
                            } else {
                                self.get_register_value(register).overflowing_sub(1)
                            };

                            self.set_register_value(register, value)?;
                            (value, overflowing)
                        }
                    };
                    self.set_flags(result as usize, get_register_size(register), overflowed);
                }
                [op @ ("inc" | "dec"), memory_address]
                    if self.memory_manager.is_memory_operand(memory_address) =>
                {
                    // Case with WORD/BYTE PTR

                    let (size_option, memory_address_str) = self.get_argument_size(memory_address);
                    let inc = *op == "inc";
                    let size = size_option.unwrap_or(VariableSize::Byte);
                    // Calculate effective address
                    match self.memory_manager.calculate_effective_address(
                        memory_address_str,
                        &self.registers,
                        true,
                    ) {
                        Ok(parsed_address) => {
                            let (current, overflowed) = match size {
                                VariableSize::Byte => {
                                    if inc {
                                        let (cu8, o) = self
                                            .memory_manager
                                            .get_byte(parsed_address)?
                                            .overflowing_add(1);
                                        (cu8 as u32, o)
                                    } else {
                                        let (cu8, o) = self
                                            .memory_manager
                                            .get_byte(parsed_address)?
                                            .overflowing_sub(1);
                                        (cu8 as u32, o)
                                    }
                                }
                                VariableSize::Word => {
                                    if inc {
                                        let (cu16, o) = self
                                            .memory_manager
                                            .get_word(parsed_address)?
                                            .overflowing_add(1);
                                        (cu16 as u32, o)
                                    } else {
                                        let (cu16, o) = self
                                            .memory_manager
                                            .get_word(parsed_address)?
                                            .overflowing_sub(1);
                                        (cu16 as u32, o)
                                    }
                                }
                                VariableSize::DoubleWord => {
                                    if inc {
                                        let (cu16, o) = self
                                            .memory_manager
                                            .get_dword(parsed_address)?
                                            .overflowing_add(1);
                                        (cu16, o)
                                    } else {
                                        let (cu16, o) = self
                                            .memory_manager
                                            .get_dword(parsed_address)?
                                            .overflowing_sub(1);
                                        (cu16, o)
                                    }
                                }
                            };

                            self.set_flags(current as usize, size, overflowed);

                            match size {
                                VariableSize::Byte => self
                                    .memory_manager
                                    .set_byte(parsed_address, current as u8)?,
                                VariableSize::Word => self
                                    .memory_manager
                                    .set_word(parsed_address, current as u16)?,
                                VariableSize::DoubleWord => self
                                    .memory_manager
                                    .set_dword(parsed_address, current)?,
                            };
                        }
                        Err(error) => return Err(error),
                    }
                }
                ["inc", _rest @ ..] => {
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }
                    println!("{}", Instruction::get_help_string(Instruction::Inc));
                    return Err(ErrorCode::InvalidOpcode(line.join(", ")));
                }
                ["dec", _rest @ ..] => {
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }
                    println!("{}", Instruction::get_help_string(Instruction::Dec));
                    return Err(ErrorCode::InvalidOpcode(line.join(", ")));
                }
                // LEA
                ["lea", register, memory_address]
                    if RegisterName::is_valid_name(register)
                        && self.memory_manager.is_memory_operand(memory_address) =>
                {
                    let register = &RegisterName::from_str_to_reg_name(register).unwrap();
                    match self.memory_manager.calculate_effective_address(
                        memory_address,
                        &self.registers,
                        true,
                    ) {
                        Ok(parsed_address) => {
                            // Determine the register size
                            match get_register_size(register) {
                                VariableSize::Byte => {
                                    return Err(ErrorCode::NotEnoughSpace(format!(
                                        "Cannot store pointer in 1-byte register: {:?}.",
                                        register
                                    )));
                                }
                                VariableSize::Word | VariableSize::DoubleWord => {
                                    self.mov_reg_const(register, parsed_address as u32)?;
                                }
                            }
                        }
                        Err(error) => return Err(error),
                    }
                }
                ["lea", _rest @ ..] => {
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }
                    println!("{}", Instruction::get_help_string(Instruction::Lea));
                    return Err(ErrorCode::InvalidOpcode(line.join(", ")));
                }
                // MOV Instructions
                // OP    REG      MEM/REG/CONST
                ["mov", reg, parameter] if RegisterName::is_valid_name(reg) => {
                    let register = &RegisterName::from_str_to_reg_name(reg).unwrap();
                    let (size_option, memory_address) = self.get_argument_size(parameter);
                    let (constant, assumed_size) =
                        self.parse_value_from_parameter(memory_address, size_option)?;

                    let is_immediate: bool = parse_string_to_usize(parameter).is_some();

                    let reg_size = get_register_size(register);

                    let assumed_size_v = assumed_size.value();
                    let reg_size_v = reg_size.value();
                    let invalid = if is_immediate {
                        assumed_size_v > reg_size_v
                    } else {
                        assumed_size_v != reg_size_v
                    };
                    if invalid {
                        return Err(ErrorCode::InvalidValue(format!("Source {parameter} of size ({assumed_size_v}) bytes bytes and destination {reg} of size ({reg_size_v}) bytes are not compatible")));
                    }
                    self.mov_reg_const(register, constant)?;
                }
                // OP       MEM               REG/CONST
                ["mov", memory_address, parameter] => {
                    if self.memory_manager.is_memory_operand(parameter) {
                        return Err(ErrorCode::InvalidValue("Direct memory transfer is not supported.".to_string()));
                    }
                    let (size_option_src, _) = self.get_argument_size(parameter);

                    let (size_option_dst, sliced_memory_string) =
                        self.get_argument_size(memory_address);

                    let is_immediate = parse_string_to_usize(parameter).is_some();
                    if let Some(size_dst) = size_option_dst {
                        if let Some(size_src) = size_option_src {
                            let message = format!("Source {parameter} of size ({}) bytes bytes and destination {memory_address} of size ({}) bytes are not compatible", size_src.value(), size_dst.value());
                            let is_invalid = if is_immediate {
                                size_dst.value() < size_src.value()
                            } else {
                                size_dst != size_src
                            };
                            if is_invalid {
                                return Err(ErrorCode::InvalidValue(message));
                            }
                        }
                    };

                    // Calculate effective address of destination
                    match self.memory_manager.calculate_effective_address(
                        sliced_memory_string,
                        &self.registers,
                        true,
                    ) {
                        // Destination is valid address
                        Ok(parsed_address) => {
                            // Get value and size of memory to mov into destination address
                            let (constant, _) = {
                                let (v, s) =
                                    self.parse_value_from_parameter(parameter, size_option_src)?;
                                (v, size_option_src.unwrap_or(s))
                            };
                            let (_, assumed_size_memory) = self.parse_value_from_parameter(
                                sliced_memory_string,
                                size_option_dst,
                            )?;

                            match assumed_size_memory {
                                VariableSize::Byte => self
                                    .memory_manager
                                    .set_byte(parsed_address, constant as u8)?,
                                VariableSize::Word => self
                                    .memory_manager
                                    .set_word(parsed_address, constant as u16)?,
                                VariableSize::DoubleWord => self
                                    .memory_manager
                                    .set_dword(parsed_address, constant)?,
                            };
                        }
                        // Destination is not valid address
                        Err(error) => return Err(error),
                    }
                }
                // HELP COMMAND
                ["mov", _rest @ ..] => {
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }
                    println!("{}", Instruction::get_help_string(Instruction::Mov));
                    return Err(ErrorCode::InvalidOpcode(line.join(", ")));
                }
                // ADD/SUB Instructions
                //         OP              REG          MEM/REG/CONST
                [op @ ("add" | "sub"), register, parameter]
                    if RegisterName::is_valid_name(register) =>
                {
                    // Parse the register name
                    let register = &RegisterName::from_str_to_reg_name(register).unwrap();

                    // Determine if the operation is addition or subtraction
                    let is_addition = *op == "add";
                    // Check if the parameter is an immediate value
                    let is_immediate = parse_string_to_usize(parameter).is_some();

                    // Get the argument size and trimmed parameter
                    let (size_option, trimmed_parameter) = self.get_argument_size(parameter);

                    // Parse the value from the parameter and get its size
                    let (constant, assumed_size) =
                        self.parse_value_from_parameter(trimmed_parameter, size_option)?;

                    // Get the size of the register
                    let reg_size = get_register_size(register);

                    // Convert sizes to their numerical values for comparison
                    let assumed_size_v = assumed_size.value();
                    let reg_size_v = reg_size.value();

                    // Check for size compatibility between the source and destination
                    let invalid = if is_immediate {
                        assumed_size_v > reg_size_v
                    } else {
                        assumed_size_v != reg_size_v
                    };

                    // Return an error if sizes are incompatible
                    if invalid {
                        return Err(ErrorCode::InvalidValue(format!(
                            "Source {} of size ({}) bytes and destination {:?} of size ({}) bytes are not compatible",
                            parameter, assumed_size_v, register, reg_size_v)));
                    }

                    // Perform the add or sub operation
                    self.add_or_sub_reg_const(register, constant, is_addition)?;
                }
                //         OP               MEM          REG/CONST
                [op @ ("add" | "sub"), memory_address, parameter] => {
                    let is_addition = *op == "add";
                    let is_immediate: bool = parse_string_to_usize(parameter).is_some();

                    if self.memory_manager.is_memory_operand(parameter) {
                        return Err(ErrorCode::InvalidValue("Direct memory transfer is not supported.".to_string()));
                    }
                    // Throwing away the second value because we don't need to trim it.
                    // The second parameter cannot be a memory operand anyway.
                    let (size_option_src, _) = self.get_argument_size(parameter);
                    let (size_option_dest, memory_address_str_dest) =
                        self.get_argument_size(memory_address);

                    // Validate size compatibility if both source and destination sizes are specified
                    if let (Some(src_size), Some(dest_size)) = (size_option_src, size_option_dest) {
                        let src_size_val = src_size.value();
                        let dest_size_val = dest_size.value();

                        let invalid = if is_immediate {
                            src_size_val > dest_size_val
                        } else {
                            src_size_val != dest_size_val
                        };

                        if invalid {
                            return Err(ErrorCode::InvalidValue(format!(
                                "Source {parameter} of size ({src_size_val}) bytes and destination {memory_address} of size ({dest_size_val}) bytes are not compatible"
                            )));
                        }
                    }

                    // Calculate effective address of the destination
                    match self.memory_manager.calculate_effective_address(
                        memory_address_str_dest,
                        &self.registers,
                        true,
                    ) {
                        // If the destination is a valid address
                        Ok(parsed_address) => {
                            // Parse the value from the parameter and get its size
                            let (constant, assumed_size) =
                                self.parse_value_from_parameter(parameter, size_option_src)?;

                            // Save the current value of EAX
                            let eax: u32 = self.get_register_value(&RegisterName::EAX);
                            // Load the constant into EAX
                            self.registers[RegisterName::EAX.to_index()].load_dword(constant);

                            // Perform the add or sub operation based on the size
                            match assumed_size {
                                VariableSize::Byte => self.add_or_sub_mem_reg(
                                    parsed_address,
                                    &RegisterName::AL,
                                    is_addition,
                                )?,
                                VariableSize::Word => self.add_or_sub_mem_reg(
                                    parsed_address,
                                    &RegisterName::AX,
                                    is_addition,
                                )?,
                                VariableSize::DoubleWord => self.add_or_sub_mem_reg(
                                    parsed_address,
                                    &RegisterName::EAX,
                                    is_addition,
                                )?,
                            };

                            // Restore the original value of EAX
                            self.registers[RegisterName::EAX.to_index()].load_dword(eax);
                        }
                        // If the destination is not a valid address, return the error
                        Err(error) => return Err(error),
                    }
                }
                ["add", _rest @ ..] => {
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }
                    println!("{}", Instruction::get_help_string(Instruction::Add));
                    return Err(ErrorCode::InvalidOpcode(line.join(", ")));
                }
                ["sub", _rest @ ..] => {
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }
                    println!("{}", Instruction::get_help_string(Instruction::Sub));
                    return Err(ErrorCode::InvalidOpcode(line.join(", ")));
                }
                // MULL / IMUL
                [op @ ("mul" | "imul"), parameter] => {
                    // Determine size of the operand
                    let (size_option_src, memory_address_str_src) =
                        self.get_argument_size(parameter);

                    let (src_value, backup_size) =
                        self.parse_value_from_parameter(memory_address_str_src, None)?;

                    let size = size_option_src.unwrap_or(backup_size);

                    self.mul_value(src_value, size, *op == "imul")?;
                }
                [op @ ("mul" | "imul"), _rest @ ..] => {
                    // Determine if the operation is signed multiplication (IMUL) or unsigned multiplication (MUL)
                    let signed = *op == "imul";

                    // Printing purposes
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }

                    // Print the appropriate help string based on the operation
                    if signed {
                        println!("{}", Instruction::get_help_string(Instruction::Imul));
                    } else {
                        println!("{}", Instruction::get_help_string(Instruction::Mul));
                    }

                    // Return an error indicating the invalid opcode
                    return Err(ErrorCode::InvalidOpcode(line.join(", ")));
                }
                // DIV / IDIV Instructions
                [op @ ("div" | "idiv"), parameter] => {
                    // Determine size of the operand
                    let (size_option_src, memory_address_str_src) =
                        self.get_argument_size(parameter);

                    let (src_value, backup_size) =
                        self.parse_value_from_parameter(memory_address_str_src, None)?;

                    let size = size_option_src.unwrap_or(backup_size);

                    self.div_value(src_value, size, *op == "idiv")?;
                }
                [op @ ("div" | "idiv"), _rest @ ..] => {
                    // Determine if the operation is signed (IDIV) or unsigned (DIV)
                    let signed = *op == "idiv";

                    // Skip the current line as it's not valid
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }

                    // Print the appropriate help string based on the operation
                    if signed {
                        println!("{}", Instruction::get_help_string(Instruction::Idiv));
                    } else {
                        println!("{}", Instruction::get_help_string(Instruction::Div));
                    }

                    // Return an error indicating the invalid opcode
                    return Err(ErrorCode::InvalidOpcode(line.join(", ")));
                }
                // Handle SHR and SHL instructions
                //         OP              REG          PARAMETER
                [op @ ("shr" | "shl"), register, parameter]
                    if RegisterName::is_valid_name(register) =>
                {
                    // Determine if the shift amount is an immediate value or 'CL'
                    let is_immediate: bool = parse_string_to_usize(parameter).is_some();

                    // Get the register name from the string
                    let register = &RegisterName::from_str_to_reg_name(register).unwrap();

                    // Determine if the shift amount is 'CL' (using the count in CL register) or an immediate value
                    let is_cl = *parameter == "CL";
                    let shift_amount = if is_cl {
                        // Use the value from CL register if shift count is 'CL'
                        self.get_register_value(&RegisterName::CL) as u8
                    } else if is_immediate {
                        // Parse the immediate value
                        parse_string_to_usize(parameter).unwrap() as u8
                    } else {
                        // Error: Parameter must be an immediate value or 'CL'
                        return Err(ErrorCode::InvalidValue(format!(
                            "Parameter {} can only be an immediate value, or 'CL'",
                            parameter
                        )));
                    };

                    // Get the size of the register operand
                    let register_size = get_register_size(register);

                    // Determine if it's a shift right (SHR) or shift left (SHL)
                    let is_shr = *op == "shr";

                    // Perform the shift operation based on the register size
                    match register_size {
                        VariableSize::Byte => {
                            let value_masked = shift_amount & 0b111; // Mask to 3 bits (0-7)
                            if shift_amount > 7 {
                                println!(
                                    "[WARNING] Shift Amount is truncated to 3 bits: ({}).",
                                    value_masked
                                );
                            }
                            // Confirmed to be byte sized by earlier if/match
                            let top = register.is_top().unwrap();
                            let current_value: u8 =
                                self.registers[register.to_index()].get_byte(top);
                            let carry_flag = (current_value >> (value_masked - 1)) & 1; // Last bit shifted out

                            let new_value = if is_shr {
                                current_value >> value_masked
                            } else {
                                current_value << value_masked
                            };

                            self.registers[register.to_index()].load_byte(new_value, top);
                            self.set_flags(new_value as usize, VariableSize::Byte, false);
                            self.set_flag(Flag::Carry, carry_flag == 1);
                        }
                        VariableSize::Word => {
                            let value_masked = shift_amount & 0b11111; // Mask to 5 bits (0-31)
                            if shift_amount > 31 {
                                println!(
                                    "[WARNING] Shift Amount is truncated to 5 bits: ({}).",
                                    value_masked
                                );
                            }

                            let current_value: u16 = self.registers[register.to_index()].get_word();
                            let carry_flag = (current_value >> (value_masked - 1)) & 1; // Last bit shifted out

                            let new_value = if is_shr {
                                current_value >> value_masked
                            } else {
                                current_value << value_masked
                            };

                            self.registers[register.to_index()].load_word(new_value);
                            self.set_flags(new_value as usize, VariableSize::Word, false);
                            self.set_flag(Flag::Carry, carry_flag == 1);
                        }
                        VariableSize::DoubleWord => {
                            let value_masked = shift_amount & 0b11111; // Mask to 5 bits (0-31)
                            if shift_amount > 31 {
                                println!(
                                    "[WARNING] Shift Amount is truncated to 5 bits: ({}).",
                                    value_masked
                                );
                            }

                            let current_value: u32 =
                                self.registers[register.to_index()].get_dword();
                            let carry_flag = (current_value >> (value_masked - 1)) & 1; // Last bit shifted out

                            let new_value = if is_shr {
                                current_value >> value_masked
                            } else {
                                current_value << value_masked
                            };

                            self.registers[register.to_index()].load_dword(new_value);
                            self.set_flags(new_value as usize, VariableSize::DoubleWord, false);
                            self.set_flag(Flag::Carry, carry_flag == 1);
                        }
                    }
                }
                // Handle SHR and SHL instructions with memory address operand
                [op @ ("shr" | "shl"), memory_address, parameter] => {
                    // Determine if the shift amount is an immediate value or 'CL'
                    let is_immediate: bool = parse_string_to_usize(parameter).is_some();

                    // Determine if the shift count is 'CL' or an immediate value
                    let is_cl = *parameter == "CL";
                    let shift_amount = if is_cl {
                        // Use the value from CL register if shift count is 'CL'
                        self.get_register_value(&RegisterName::CL) as u8
                    } else if is_immediate {
                        // Parse the immediate value
                        parse_string_to_usize(parameter).unwrap() as u8
                    } else {
                        // Error: Parameter must be an immediate value or 'CL'
                        return Err(ErrorCode::InvalidValue(format!(
                            "Parameter {} can only be an immediate value, or 'CL'",
                            parameter
                        )));
                    };

                    // Get the size of the destination operand
                    let (size_option_dst, trimmed_dst) = self.get_argument_size(memory_address);
                    let (memory_value, size_dst) =
                        self.parse_value_from_parameter(trimmed_dst, size_option_dst)?;

                    // Calculate the effective address of the destination memory operand
                    let destination = self.memory_manager.calculate_effective_address(
                        trimmed_dst,
                        &self.registers,
                        true,
                    )?;

                    // Determine if it's a shift right (SHR) or shift left (SHL)
                    let is_shr = *op == "shr";

                    // Perform the shift operation based on the size of the destination operand
                    match size_dst {
                        VariableSize::Byte => {
                            let value_masked = shift_amount & 0b111; // Mask to 3 bits (0-7)
                            if shift_amount > 7 {
                                println!(
                                    "[WARNING] Shift Amount is truncated to 3 bits: ({}).",
                                    value_masked
                                );
                            }

                            let current_value = memory_value as u8;
                            let carry_flag = (current_value >> (value_masked - 1)) & 1; // Last bit shifted out

                            let new_value = if is_shr {
                                current_value >> value_masked
                            } else {
                                current_value << value_masked
                            };

                            // Update the memory with the shifted value
                            self.memory_manager.set_byte(destination, new_value)?;
                            self.set_flags(new_value as usize, VariableSize::Byte, false);
                            self.set_flag(Flag::Carry, carry_flag == 1);
                        }
                        VariableSize::Word => {
                            let value_masked = shift_amount & 0b11111; // Mask to 5 bits (0-31)
                            if shift_amount > 31 {
                                println!(
                                    "[WARNING] Shift Amount is truncated to 5 bits: ({}).",
                                    value_masked
                                );
                            }

                            let current_value = memory_value as u16;
                            let carry_flag = (current_value >> (value_masked - 1)) & 1; // Last bit shifted out

                            let new_value = if is_shr {
                                current_value >> value_masked
                            } else {
                                current_value << value_masked
                            };

                            // Update the memory with the shifted value
                            self.memory_manager.set_word(destination, new_value)?;
                            self.set_flags(new_value as usize, VariableSize::Word, false);
                            self.set_flag(Flag::Carry, carry_flag == 1);
                        }
                        VariableSize::DoubleWord => {
                            let value_masked = shift_amount & 0b11111; // Mask to 5 bits (0-31)
                            if shift_amount > 31 {
                                println!(
                                    "[WARNING] Shift Amount is truncated to 5 bits: ({}).",
                                    value_masked
                                );
                            }

                            let current_value = memory_value;
                            let carry_flag = (current_value >> (value_masked - 1)) & 1; // Last bit shifted out

                            let new_value = if is_shr {
                                current_value >> value_masked
                            } else {
                                current_value << value_masked
                            };

                            // Update the memory with the shifted value
                            self.memory_manager.set_dword(destination, new_value)?;
                            self.set_flag(Flag::Carry, carry_flag == 1);
                        }
                    }
                }
                // PRINT  Instructions
                ["print", parameter] => {
                    let args: Vec<&str> = parameter.split_whitespace().collect();
                    let trimmed_parameter = if args[0] == "char" {
                        args[1..].join(" ")
                    } else {
                        args.join(" ")
                    };
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }
                    if let Some((start_char, end_char)) =
                        parameter.chars().next().zip(parameter.chars().next_back())
                    {
                        if start_char == '\'' && end_char == '\'' {
                            println!("[PRINT]@[IP={ip}]:\t{parameter}\n");
                            continue;
                        }
                    }

                    let (size, memory_address_str_src) = self.get_argument_size(&trimmed_parameter);
                    let (src_value, _) =
                        self.parse_value_from_parameter(memory_address_str_src, size)?;
                    if args.into_iter().nth(2) == Some("char") {
                        if let Some(src_value_char) = std::char::from_u32(src_value) {
                            println!("[PRINT]@[IP={ip}] {parameter}: {0}\n", src_value_char);
                        }
                    } else {
                        // Check if argument is a string literal and remove surrounding quotes
                        println!("[PRINT]@[IP={ip}] {parameter}: {0}\n", src_value);
                    }
                }
                ["print", number, memory_address_maybe_ch] => {
                    //if self.memory_manager.is_memory_operand(memory_address) &&
                    //parse_string_to_usize(*number).is_some() => {

                    let args: Vec<&str> = memory_address_maybe_ch.split_whitespace().collect();

                    let (ch, memory_address) = match args.as_slice() {
                        ["char", address] => (true, *address),
                        [address] => (false, *address),
                        _ => return Err(ErrorCode::InvalidOpcode(line.join(", "))),
                    };

                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }

                    let (size_option_src, trimmed_address) = self.get_argument_size(memory_address);
                    let size = size_option_src.unwrap_or(VariableSize::Byte);
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(
                        trimmed_address,
                        &self.registers,
                        true,
                    ) {
                        let (value, _) = self.parse_value_from_parameter(number, None)?;
                        self.memory_manager.check_memory_address(
                            parsed_address + (value as usize) * size.value(),
                        )?;
                        let ip = self.registers[ip_index].get_word();
                        print!(
                            "[PRINT]@[IP={ip}][{parsed_address}..{}]:\t[",
                            parsed_address + (value as usize) - 1
                        );
                        for i in 0..value {
                            match size {
                                VariableSize::Byte => {
                                    let src_value = self
                                        .memory_manager
                                        .get_byte(parsed_address + (i as usize) * size.value())?;

                                    if ch {
                                        if let Some(src_value_char) =
                                            std::char::from_u32(src_value as u32)
                                        {
                                            print!("{0}", src_value_char);
                                        } else {
                                            print!("{: >width$}", src_value, width = 4);
                                        }
                                    } else {
                                        print!("{: >width$}", src_value, width = 4);
                                    }
                                }
                                VariableSize::Word => {
                                    let src_value = self
                                        .memory_manager
                                        .get_word(parsed_address + (i as usize) * size.value())?;
                                    if ch {
                                        if let Some(src_value_char) =
                                            std::char::from_u32(src_value as u32)
                                        {
                                            print!("{0}", src_value_char);
                                        } else {
                                            print!(
                                                "{: >width$}", src_value, width = 4
                                            );
                                        }
                                    } else {
                                        print!("{: >width$}", src_value, width = 4);
                                    }
                                }
                                VariableSize::DoubleWord => {
                                    let src_value = self
                                        .memory_manager
                                        .get_dword(parsed_address + (i as usize) * size.value())?;
                                    if ch {
                                        if let Some(src_value_char) = std::char::from_u32(src_value)
                                        {
                                            print!("{0}", src_value_char);
                                        } else {
                                            print!("{0} ", src_value);
                                        }
                                    } else {
                                        print!("{0} ", src_value);
                                    }
                                }
                            }
                        }
                        println!("]");

                    // } else {
                    //     return Err(ErrorCode::InvalidValue(format!("Integer {number} could not be parsed.")))
                    // }
                    } else {
                        return Err(ErrorCode::InvalidValue(format!(
                            "Memory address {memory_address} could not be parsed."
                        )));
                    }
                }
                ["NOP"] => {}
                [variable_name, define_as @ ("db" | "dw" | "dd"), rest @ ..]
                    if self.memory_manager.is_valid_variable_name(variable_name) =>
                {
                    // (parse_string_to_usize(*data).is_some() || self.memory_manager.is_valid_array(*data).is_ok() )=> {
                    let size: VariableSize = match *define_as {
                        "db" => VariableSize::Byte,
                        "dw" => VariableSize::Word,
                        "dd" => VariableSize::DoubleWord,
                        _ => {
                            return Err(ErrorCode::InvalidValue(
                                "Invalid Variable Size".to_string(),
                            ))
                        }
                    };
                    let mut bytes: Vec<u32> = Vec::new();
                    for &arg in rest.iter() {
                        // Check if argument is a string literal and remove surrounding quotes
                        if let Some((start_char, end_char)) =
                            arg.chars().next().zip(arg.chars().next_back())
                        {
                            if (start_char == '"' && end_char == '"')
                                || (start_char == '\'' && end_char == '\'')
                            {
                                let inner = &arg[1..arg.len() - 1];
                                for c in inner.chars() {
                                    bytes.push(c as u32);
                                }
                            } else {
                                // Handle other cases (numeric values, etc.)
                                if let Some(value) = parse_string_to_usize(arg) {
                                    bytes.push(value);
                                } else {
                                    return Err(ErrorCode::InvalidValue(format!(
                                        "Could not parse {arg}"
                                    )));
                                }
                            }
                        } else {
                            // Handle other cases (numeric values, etc.)
                            if let Some(value) = parse_string_to_usize(arg) {
                                bytes.push(value);
                            } else {
                                return Err(ErrorCode::InvalidValue(format!(
                                    "Could not parse {arg}"
                                )));
                            }
                        }
                    }
                    if debug {
                        lines_to_skip += 1;
                        let _ = skip_lines(lines_to_skip);
                    }
                    self.memory_manager
                            .save_variable(variable_name.to_string(), &bytes, size)?
                }
                //////// JUMPS ////////////
                ["jmp", label] => {
                    if let Err(error) = self.jump_to(label) {
                        if debug {
                            lines_to_skip += 1;
                            let _ = skip_lines(lines_to_skip);
                        }
                        println!("{}", Instruction::get_help_string(Instruction::Jmp));
                        return Err(error);
                    }
                }
                // JUMPS
                [_flag @ ("je" | "jz" | "jne" | "jnz"), label] => {
                    let equal = *_flag == "je" || *_flag == "jz";
                    if equal != self.is_flag_on(Flag::Zero) {
                        continue;
                    }
                    if let Err(error) = self.jump_to(label) {
                        if debug {
                            lines_to_skip += 1;
                            let _ = skip_lines(lines_to_skip);
                        }
                        match *_flag {
                            "je" => println!("{}", Instruction::get_help_string(Instruction::Je)),
                            "jz" => println!("{}", Instruction::get_help_string(Instruction::Jz)),
                            "jne" => println!("{}", Instruction::get_help_string(Instruction::Jne)),
                            "jnz" => println!("{}", Instruction::get_help_string(Instruction::Jnz)),
                            _ => {}
                        }
                        return Err(error);
                    }
                }
                [_flag @ ("jg" | "jge"), label] => {
                    let is_jg = *_flag == "jg";

                    if is_jg && self.is_flag_on(Flag::Zero) {
                        continue;
                    }
                    if self.is_flag_on(Flag::Sign) != self.is_flag_on(Flag::Overflow) {
                        continue;
                    }

                    if let Err(error) = self.jump_to(label) {
                        if debug {
                            lines_to_skip += 1;
                            let _ = skip_lines(lines_to_skip);
                        }
                        match *_flag {
                            "jg" => println!("{}", Instruction::get_help_string(Instruction::Jg)),
                            "jge" => println!("{}", Instruction::get_help_string(Instruction::Jge)),
                            _ => {}
                        }
                        return Err(error);
                    }
                }
                [_flag @ ("jl" | "jle"), label] => {
                    let is_jl = *_flag == "jl";

                    if is_jl && self.is_flag_on(Flag::Zero) {
                        continue;
                    }

                    if self.is_flag_on(Flag::Sign) == self.is_flag_on(Flag::Overflow) {
                        continue;
                    }

                    if let Err(error) = self.jump_to(label) {
                        if debug {
                            lines_to_skip += 1;
                            let _ = skip_lines(lines_to_skip);
                        }
                        match *_flag {
                            "jl" => println!("{}", Instruction::get_help_string(Instruction::Jl)),
                            "jle" => println!("{}", Instruction::get_help_string(Instruction::Jle)),
                            _ => {}
                        }
                        return Err(error);
                    }
                }
                [_flag @ ("ja" | "jae"), label] => {
                    let is_ja = *_flag == "ja";

                    if self.is_flag_on(Flag::Carry) || (is_ja && self.is_flag_on(Flag::Zero)) {
                        continue;
                    }

                    if let Err(error) = self.jump_to(label) {
                        if debug {
                            lines_to_skip += 1;
                            let _ = skip_lines(lines_to_skip);
                        }
                        match *_flag {
                            "ja" => println!("{}", Instruction::get_help_string(Instruction::Ja)),
                            "jae" => println!("{}", Instruction::get_help_string(Instruction::Jae)),
                            _ => {}
                        }
                        return Err(error);
                    }
                }
                [_flag @ ("jb" | "jbe"), label] => {
                    let is_jb = *_flag == "jb";

                    if self.is_flag_on(Flag::Carry) || (is_jb && self.is_flag_on(Flag::Zero)) {
                        continue;
                    }

                    if let Err(error) = self.jump_to(label) {
                        if debug {
                            lines_to_skip += 1;
                            let _ = skip_lines(lines_to_skip);
                        }
                        match *_flag {
                            "jb" => println!("{}", Instruction::get_help_string(Instruction::Jb)),
                            "jbe" => println!("{}", Instruction::get_help_string(Instruction::Jbe)),
                            _ => {}
                        }
                        return Err(error);
                    }
                }
                // CMP
                ["cmp", first_operand, second_operand] => {
                    let (first_size_option, trimmed_first_parameter) =
                        self.get_argument_size(first_operand);
                    let (first_operand_value, first_operand_size) = self
                        .parse_value_from_parameter(trimmed_first_parameter, first_size_option)?;

                    let (second_size_option, trimmed_second_parameter) =
                        self.get_argument_size(second_operand);
                    let (second_operand_value, second_operand_size) = self
                        .parse_value_from_parameter(trimmed_second_parameter, second_size_option)?;

                    let second_is_immediate: bool = parse_string_to_usize(second_operand).is_some();

                    if (second_is_immediate && first_operand_size.value() < second_operand_size.value()) 
                        || (!second_is_immediate && first_operand_size != second_operand_size) {
                        
                        return Err(ErrorCode::InvalidValue(
                            format!(
                                "Target memory pointer size ({}) bytes doesn't match second parameter size ({}) bytes",
                                first_operand_size.value(), 
                                second_operand_size.value()
                            )
                        ));
                    }
                    // Initialize variables to store parsed values
                    let first_value = first_operand_value as isize;
                    let second_value = second_operand_value as isize;

                    let result = first_value - second_value;
                    // Set flags based on comparison results
                    self.set_flag(Flag::Zero, result == 0);
                    self.set_flag(Flag::Carry, first_value < second_value);
                    self.set_flag(
                        Flag::Overflow,
                        (first_value < 0 && second_value > 0 && result > 0)
                            || (first_value > 0 && second_value < 0 && result < 0),
                    );
                    self.set_flag(Flag::Sign, result < 0);
                    self.set_flag(Flag::Parity, result.count_ones() % 2 == 0);
                }
                ["call", label] => {
                    let ip: u32 = self.get_register_value(&RegisterName::IP);
                    match self.jump_to(label) {
                        Ok(_) => {
                            self.memory_manager.push_to_stack(
                                ip,
                                VariableSize::Word,
                                &mut self.registers[RegisterName::ESI.to_index()],
                            )?;
                        }
                        Err(error) => return Err(error),
                    }
                }
                ["ret"] => {
                    let ip_from_stack = self.memory_manager.pop_from_stack(
                        VariableSize::Word,
                        &mut self.registers[RegisterName::ESI.to_index()],
                    )?;
                    // I would use JUMP_TO but it hates me apparently.
                    self.registers[ip_index].load_word(ip_from_stack as u16);
                    // self.lines.set_ip(ip_from_stack as usize);
                }
                // STACK OPERATIONS
                ["push", parameter] => {
                    // No "WORD PTR" etc.

                    let (size_option, trimmed_parameter) = self.get_argument_size(parameter);

                    let (value, size) =
                        self.parse_value_from_parameter(trimmed_parameter, size_option)?;
                    self.memory_manager.push_to_stack(
                        value,
                        size,
                        &mut self.registers[RegisterName::SI.to_index()],
                    )?;
                }
                ["pop", parameter] => {
                    // No "WORD PTR" etc.
                    let (size_option, trimmed_parameter) = self.get_argument_size(parameter);

                    let (_, size) =
                        self.parse_value_from_parameter(trimmed_parameter, size_option)?;
                    let popped_value = self
                        .memory_manager
                        .pop_from_stack(size, &mut self.registers[RegisterName::SI.to_index()])?;
                    if let Ok(register_name) = RegisterName::from_str_to_reg_name(parameter) {
                        match size {
                            VariableSize::Byte => {
                                return Err(ErrorCode::InvalidValue(
                                    "POP can only receive a 16-bit or 32-bit parameter."
                                        .to_string(),
                                ))
                            }
                            VariableSize::Word => self.registers[register_name.to_index()]
                                .load_word(popped_value as u16),
                            VariableSize::DoubleWord => {
                                self.registers[register_name.to_index()].load_dword(popped_value)
                            }
                        }
                    } else {
                        // Calculate effective address of destination
                        match self.memory_manager.calculate_effective_address(
                            trimmed_parameter,
                            &self.registers,
                            true,
                        ) {
                            // Destination is valid address
                            Ok(parsed_address) => match size {
                                VariableSize::Byte => {
                                    return Err(ErrorCode::InvalidValue(
                                        "POP can only receive a 16-bit or 32-bit parameter."
                                            .to_string(),
                                    ))
                                }
                                VariableSize::Word => self
                                    .memory_manager
                                    .set_word(parsed_address, popped_value as u16)?,
                                VariableSize::DoubleWord => self
                                    .memory_manager
                                    .set_dword(parsed_address, popped_value)?,
                            },
                            Err(error) => return Err(error),
                        }
                    }
                }
                // IGNORE LABELS
                [label] if label.ends_with(':') => {
                    let no_colon = &label[..label.len() - 1];
                    // Found no label
                    if !self.memory_manager.labels.contains_key(no_colon) {
                        let error_msg = format!(
                            "Unknown instruction: {:?}.\nPerhaps you misspelt the label name?",
                            label
                        );
                        return Err(ErrorCode::InvalidOpcode(error_msg));
                    }
                }
                // SKIP PROCS
                [proc, arg @ ("PROC" | "END")] => {
                    let end = *arg == "END";

                    let end_ip = if let Some((_, end_ip)) = self.memory_manager.procs.get(*proc) {
                        *end_ip
                    } else {
                        let error_msg: String = format!(
                            "Unknown instruction: {:?}.\nPerhaps you misspelt the proc name?",
                            proc
                        );
                        return Err(ErrorCode::InvalidOpcode(error_msg));
                    };

                    if end {
                        let error_msg: String = "Must return in proc.".to_string();
                        return Err(ErrorCode::InvalidOpcode(error_msg));
                    } else {
                        let caller_ip = format!("{}", end_ip);
                        self.jump_to(&caller_ip.as_str())?;
                    }
                }

                // NO MATCH
                _ => {
                    let error_msg = format!("Unknown instruction: {:?}", line);
                    return Err(ErrorCode::InvalidOpcode(error_msg));
                    // Handle unrecognized instructions
                }
            }
            // Won't panic
            // If some JUMP was made, this will update the lines module.
            // If no JUMP was made, this basically does nothing, as it sets lines' ip to itself.
            self.lines
                .set_ip(self.get_register_value(&RegisterName::IP) as usize);
        }
        for _ in 0..lines_to_skip {
            println!();
        }
        Ok(())
    }

    fn jump_to(&mut self, label: &&str) -> Result<usize, ErrorCode> {
        let ip_index = RegisterName::IP.to_index();
        if let Some(address) = self.memory_manager.labels.get(*label) {
            // self.lines.set_ip(*address);
            self.registers[ip_index].load_word(*address as u16);
            Ok(*address)
        } else if let Some((start_ip, _)) = self.memory_manager.procs.get(*label) {
            // self.lines.set_ip(*start_ip);
            self.registers[ip_index].load_word(*start_ip as u16);
            Ok(*start_ip)
        } else {
            let (size_option, trimmed) = self.get_argument_size(label);

            if trimmed != *label {
                return Err(ErrorCode::InvalidPointer("Invalid Syntax.".to_string()));
            }

            let target = if self.memory_manager.is_memory_operand(label) {
                self.memory_manager
                    .calculate_effective_address(label, &self.registers, true)?
            } else {
                let (a, _) = self.parse_value_from_parameter(label, size_option)?;
                a as usize
            };

            if target > MEMORY_SIZE - 1 {
                return Err(ErrorCode::InvalidPointer(format!(
                    "Target IP \"{}\" is outside of memory bounds.",
                    target
                )));
            }

            self.registers[ip_index].load_word(target as u16);
            // self.lines.set_ip(target);
            Ok(target)
        }
    }

    fn is_flag_on(&self, flag: Flag) -> bool {
        // No need to check, because we know FLAG is a valid register.
        // Also, it's 16 bits, i.e., we can convert it without issues.
        let value = self.get_register_value(&RegisterName::FLAG) as u16;
        value & flag.value() != 0
    }

    // MOV operations
    // REG <- Reg, Const, Var, Mem
    // Mem <- Reg, Const, Var
    // Var <- Reg, Const, Var
    // Const <- NOTHING, Const can't be moved into

    fn mov_reg_const(&mut self, dest: &RegisterName, constant: u32) -> Result<(), ErrorCode> {
        let index = dest.to_index();
        let size = match get_register_size(dest) {
            VariableSize::Byte => {
                if constant as i32 > u8::MAX.into() {
                    return Err(ErrorCode::InvalidValue(format!(
                        "Value {constant} cannot fit into {:?}",
                        dest
                    )));
                } // Confirmed to be byte sized by earlier if/match
                self.registers[index].load_byte(constant as u8, dest.is_top().unwrap());
                VariableSize::Byte
            }
            VariableSize::Word => {
                if constant as i32 > u16::MAX.into() {
                    return Err(ErrorCode::InvalidValue(format!(
                        "Value {constant} cannot fit into {:?}",
                        dest
                    )));
                }
                self.registers[index].load_word(constant as u16);
                VariableSize::Word
            }
            VariableSize::DoubleWord => {
                self.registers[index].load_dword(constant);
                VariableSize::DoubleWord
            }
        };
        self.set_flags(constant as usize, size, false);
        Ok(())
    }

    fn add_or_sub_mem_reg(
        &mut self,
        memory_address: usize,
        src: &RegisterName,
        is_addition: bool,
    ) -> Result<(), ErrorCode> {
        let index = src.to_index();
        let size = get_register_size(src);

        let (result, overflowed) = match size {
            VariableSize::Byte => {
                //Confirmed to be byte sized by earlier if/match
                let src_value = self.registers[index].get_byte(src.is_top().unwrap());
                let dest_value = self.memory_manager.get_byte(memory_address)?;
                let (result, overflowed) = if is_addition {
                    dest_value.overflowing_add(src_value)
                } else {
                    dest_value.overflowing_sub(src_value)
                };
                self.memory_manager.set_byte(memory_address, result)?;
                (result as u32, overflowed)
            }
            VariableSize::Word => {
                let src_value = self.registers[index].get_word();
                let dest_value = self.memory_manager.get_word(memory_address)?;
                let (result, overflowed) = if is_addition {
                    dest_value.overflowing_add(src_value)
                } else {
                    dest_value.overflowing_sub(src_value)
                };
                self.memory_manager.set_word(memory_address, result)?;
                (result as u32, overflowed)
            }
            VariableSize::DoubleWord => {
                self.memory_manager.check_memory_address(memory_address)?;
                let src_value = self.registers[index].get_dword();
                let dest_value = self.memory_manager.get_dword(memory_address)?;
                let (result, overflowed) = if is_addition {
                    dest_value.overflowing_add(src_value)
                } else {
                    dest_value.overflowing_sub(src_value)
                };
                self.memory_manager.set_dword(memory_address, result)?;
                (result, overflowed)
            }
        };

        self.set_flags(result as usize, size, overflowed);
        Ok(())
    }

    fn add_or_sub_reg_const(
        &mut self,
        dest: &RegisterName,
        constant: u32,
        is_addition: bool,
    ) -> Result<(), ErrorCode> {
        let index = dest.to_index();
        let size = get_register_size(dest);
        match size {
            VariableSize::Byte => {
                if constant > u8::MAX as u32 {
                    return Err(ErrorCode::InvalidValue(format!(
                        "Value {} can't fit in {:?}",
                        constant, dest
                    )));
                }
                let dest_value = self.get_register_value(dest) as u8;
                let (result, overflowed) = if is_addition {
                    dest_value.overflowing_add(constant as u8)
                } else {
                    dest_value.overflowing_sub(constant as u8)
                }; //Confirmed to be byte sized by earlier if/match
                self.registers[index].load_byte(result, dest.is_top().unwrap());
                self.set_flags(result as usize, VariableSize::Byte, overflowed);
            }
            VariableSize::Word => {
                if constant > u16::MAX as u32 {
                    return Err(ErrorCode::InvalidValue(format!(
                        "Value {} can't fit in {:?}",
                        constant, dest
                    )));
                }
                let dest_value = self.get_register_value(dest) as u16;
                let (result, overflowed) = if is_addition {
                    dest_value.overflowing_add(constant as u16)
                } else {
                    dest_value.overflowing_sub(constant as u16)
                };
                self.registers[index].load_word(result);
                self.set_flags(result as usize, VariableSize::Word, overflowed);
            }
            VariableSize::DoubleWord => {
                let dest_value = self.get_register_value(dest);
                let (result, overflowed) = if is_addition {
                    dest_value.overflowing_add(constant)
                } else {
                    dest_value.overflowing_sub(constant)
                };
                self.registers[index].load_dword(result);
                self.set_flags(result as usize, VariableSize::DoubleWord, overflowed);
            }
        }

        Ok(())
    }

    fn set_register_value(
        &mut self,
        register_name: &RegisterName,
        value: u32,
    ) -> Result<(), ErrorCode> {
        let index = register_name.to_index();
        let size = get_register_size(register_name);

        match size {
            VariableSize::Byte => {
                if value > u8::MAX as u32 {
                    return Err(ErrorCode::InvalidValue(format!(
                        "Value {value} can't fit in {:?}",
                        register_name
                    )));
                } // Confirmed to be byte sized by earlier if/match
                self.registers[index]
                    .load_byte(value.try_into().unwrap(), register_name.is_top().unwrap());
            }
            VariableSize::Word => {
                if value > u16::MAX as u32 {
                    return Err(ErrorCode::InvalidValue(format!(
                        "Value {value} can't fit in {:?}",
                        register_name
                    )));
                }
                self.registers[index].load_word(value.try_into().unwrap());
            }
            VariableSize::DoubleWord => {
                self.registers[index].load_dword(value);
            }
        }

        Ok(())
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        let index = RegisterName::FLAG.to_index();
        if value {
            let current = self.registers[index].get_word();
            self.registers[index].load_word(current | flag.value());
        } else {
            let current = self.registers[index].get_word();
            self.registers[index].load_word(current & !flag.value());
        }
    }

    //////////// MUL ////////////
    // Multiply the value in the source register by the value in AX register.

    fn mul_value(
        &mut self,
        src_value: u32,
        size: VariableSize,
        signed: bool,
    ) -> Result<(), ErrorCode> {
        match size {
            VariableSize::Byte => self.mul_8bit(src_value as u8, signed),
            VariableSize::Word => self.mul_16bit(src_value as u16, signed),
            VariableSize::DoubleWord => self.mul_32bit(src_value, signed),
        }
    }

    // Function to multiply 8-bit values and store the result in AX register.
    fn mul_8bit(&mut self, src_value: u8, signed: bool) -> Result<(), ErrorCode> {
        let al_value = self.get_register_value(&RegisterName::AX) as u8;

        let result = if signed {
            let al_signed = al_value as i8;
            let src_signed = src_value as i8;
            (al_signed as i16 * src_signed as i16) as u16
        } else {
            al_value as u16 * src_value as u16
        };

        let overflow_condition = result > 0xFF;

        self.set_flag(Flag::Carry, overflow_condition);
        self.set_flag(Flag::Overflow, overflow_condition);
        self.set_register_value(&RegisterName::AX, result as u32)?;
        Ok(())
    }

    // Function to multiply 16-bit values and store the result in DX:AX register.
    fn mul_16bit(&mut self, src_value: u16, signed: bool) -> Result<(), ErrorCode> {
        let ax_value = self.get_register_value(&RegisterName::AX);

        let result = if signed {
            let ax_signed = ax_value as i16;
            let src_signed = src_value as i16;
            (ax_signed as i32 * src_signed as i32) as u32
        } else {
            ax_value * src_value as u32
        };

        let overflow_condition = result > 0xFFFF;

        self.set_flag(Flag::Carry, overflow_condition);
        self.set_flag(Flag::Overflow, overflow_condition);
        self.set_register_value(&RegisterName::AX, result & 0xFFFF)?;
        self.set_register_value(&RegisterName::DX, result >> 16)?;
        Ok(())
    }

    // Function to multiply 32-bit values and store the result in EDX:EAX register.
    fn mul_32bit(&mut self, src_value: u32, signed: bool) -> Result<(), ErrorCode> {
        let eax_value = self.get_register_value(&RegisterName::EAX);

        let result = if signed {
            let eax_signed = eax_value as i32;
            let src_signed = src_value as i32;
            (eax_signed as i64 * src_signed as i64) as u64
        } else {
            eax_value as u64 * src_value as u64
        };

        let overflow_condition = result > 0xFFFFFFFF;

        self.set_flag(Flag::Carry, overflow_condition);
        self.set_flag(Flag::Overflow, overflow_condition);
        self.set_register_value(&RegisterName::EAX, result as u32)?;
        self.set_register_value(&RegisterName::EDX, (result >> 32) as u32)?;
        Ok(())
    }

    /// DIV ////
    /// Divide the value in the source register by the value in the AX register.
    fn div_value(
        &mut self,
        src_value: u32,
        size: VariableSize,
        signed: bool,
    ) -> Result<(), ErrorCode> {
        match size {
            VariableSize::Byte => self.div_8bit(src_value as u8, signed),
            VariableSize::Word => self.div_16bit(src_value as u16, signed),
            VariableSize::DoubleWord => self.div_32bit(src_value, signed),
        }
    }

    /// Perform 8-bit division: AX / src_value, AH = AX % src_value
    fn div_8bit(&mut self, src_value: u8, signed: bool) -> Result<(), ErrorCode> {
        let ax_value = self.get_register_value(&RegisterName::AX) as u16;

        if src_value == 0 {
            return Err(ErrorCode::DivisionByZero);
        }

        let (quotient, remainder) = if signed {
            let al_signed = (ax_value as u8) as i8;
            let src_signed = src_value as i8;
            let (q, r) = (al_signed / src_signed, al_signed % src_signed);
            (q as u8, r as u8)
        } else {
            (
                (ax_value / src_value as u16) as u8,
                (ax_value % src_value as u16) as u8,
            )
        };

        self.set_register_value(&RegisterName::AL, quotient as u32)?;
        self.set_register_value(&RegisterName::AH, remainder as u32)?;
        Ok(())
    }

    /// Perform 16-bit division: DX:AX / src_value, DX = DX:AX % src_value
    fn div_16bit(&mut self, src_value: u16, signed: bool) -> Result<(), ErrorCode> {
        let ax_value = self.get_register_value(&RegisterName::AX);
        let dx_value = self.get_register_value(&RegisterName::DX);
        let dividend = (dx_value << 16) | ax_value;

        if src_value == 0 {
            return Err(ErrorCode::DivisionByZero);
        }

        let (quotient, remainder) = if signed {
            let dividend_signed = dividend as i32;
            let src_signed = src_value as i16;
            let (q, r) = (
                dividend_signed / src_signed as i32,
                dividend_signed % src_signed as i32,
            );
            (q as u16, r as u16)
        } else {
            (
                (dividend / src_value as u32) as u16,
                (dividend % src_value as u32) as u16,
            )
        };

        self.set_register_value(&RegisterName::AX, quotient as u32)?;
        self.set_register_value(&RegisterName::DX, remainder as u32)?;
        Ok(())
    }

    /// Perform 32-bit division: EDX:EAX / src_value, EDX = EDX:EAX % src_value
    fn div_32bit(&mut self, src_value: u32, signed: bool) -> Result<(), ErrorCode> {
        let eax_value = self.get_register_value(&RegisterName::EAX);
        let edx_value = self.get_register_value(&RegisterName::EDX);

        let dividend = ((edx_value as u64) << 32) | (eax_value as u64);

        if src_value == 0 {
            return Err(ErrorCode::DivisionByZero);
        }

        let (quotient, remainder, overflowed) = if signed {
            let dividend_signed = dividend as i64;
            let src_signed = src_value as i32;

            if src_signed == -1 && dividend_signed == i64::MIN {
                return Err(ErrorCode::Overflow);
            }

            let (q, overflow1) = dividend_signed.overflowing_div(src_signed as i64);
            let (r, overflow2) = dividend_signed.overflowing_rem(src_signed as i64);
            (q as u32, r as u32, overflow1 || overflow2)
        } else {
            let (q, overflow1) = dividend.overflowing_div(src_value as u64);
            let (r, overflow2) = dividend.overflowing_rem(src_value as u64);
            (q as u32, r as u32, overflow1 || overflow2)
        };

        if overflowed {
            return Err(ErrorCode::Overflow);
        }

        self.set_register_value(&RegisterName::EAX, quotient)?;
        self.set_register_value(&RegisterName::EDX, remainder)?;
        Ok(())
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let length = self.registers.clone().into_iter().len();
        for i in 0..length {
            let register: &Register = &self.registers[i];
            let low_byte = (register.get_word() & 0xFF) as u8;
            let high_byte = ((register.get_word() >> 8) & 0xFF) as u8;
            let reg_string = format!(
                "Register {:?}:\t{}\t({:08b} {:08b})",
                register.name,
                register.get_word(),
                high_byte,
                low_byte
            );
            write!(f, "{reg_string}  ")?;

            let start_index = if i > 6 {
                MEMORY_SIZE - ((length - i) * 24)
            } else {
                i * 24
            };

            let this_line = self.memory_manager._get_memory(start_index, 24);
            write!(f, "{: <6}:", start_index)?;
            for j in 0..6 {
                write!(f, "{:02X} ", this_line[j * 4])?;
                write!(f, "{:02X} ", this_line[j * 4 + 1])?;
                write!(f, "{:02X} ", this_line[j * 4 + 2])?;
                write!(f, "{:02X}  ", this_line[j * 4 + 3],)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
