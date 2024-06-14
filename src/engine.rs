use std::{
    io,
    collections::{HashMap, HashSet},
};
use crate::{
    error_code::ErrorCode,
    flag::Flag,
    line_processor::{
        LineProcessor,
        convert_vec_string_to_vec_str
    },
    variable_metadata::{
        // VariableMetadata,
        VariableSize,
    },
    memory_manager::MemoryManager,
    register::{
        get_register, 
        Register,
        get_register_size
    },
    // status::Status,
    utils::{
        read_lines_from_file, 
        parse_string_to_usize
    },
    command::Command
};

const MEMORY_SIZE: usize = 1024 * 16; // 16 KB


pub struct Engine {
    lines: LineProcessor, // lines of source code (.txt)
    pub registers: [Register; 8], // A-D, ESI, EDI, P
    memory_manager: MemoryManager, // 16 KB bytes of memory
    // mode: bool, // false = reading data, true = reading code
    labels: HashMap<String, usize>, // labels to jump to
    // status: Status, // status: ok, error, halted,
    valid_registers: HashSet<String>,
    // interrupts: Vec<Interrupt>
}


impl Engine {
    pub fn new(file_name: &str) -> io::Result<Self> {
        let file_lines = read_lines_from_file(file_name)?;
        let my_registers: [Register; 8] = [
            Register::new("EAX"),
            Register::new("EBX"),
            Register::new("ECX"),
            Register::new("EDX"),
            Register::new("ESI"),
            Register::new("EDI"),
            Register::new("IP"),
            Register::new("FLAG"),
        ];

        let ds = 0 as usize;         // DATA SEGMENT starts at 0
        let cs = 1024 * 3 as usize;  // CODE SEGMENT starts at 3072 
        let ss: usize = MEMORY_SIZE - 1024; // STACK SEGMENT, starts at 15360 (1024*15)

        let valid_registers: HashSet<String> = [         
            "AX", "BX", "CX", "DX", "SI", "DI", "IP", "FLAG", "BP", "SP",
            "AL", "AH", "BL", "BH", "CL", "CH", "DL", "DH",
            "EAX", "EBX", "ECX", "EDX", "ESI", "EDI",  "EBP", "ESP"
        ].iter().cloned().map(String::from).collect();
            Ok(Self {
                lines: LineProcessor::new(file_lines),
                registers: my_registers,
                memory_manager: MemoryManager::new(MEMORY_SIZE, [ds, cs, ss]),
                labels: HashMap::new(),
                valid_registers,
            })
    }

    fn is_valid_register(&self, reg_to_check: &str) -> bool {
        self.valid_registers.contains(reg_to_check)
    }

    pub fn get_register_value(&self, name: &str) -> Result<u32, ErrorCode> {
        if !self.is_valid_register(name) {
            return Err(ErrorCode::InvalidRegister);
        }
        let value = self.registers[get_register(name)].get_dword();
        match name {
            "AL" | "BL" | "CL" | "DL" => Ok(value & 0x000000FF),
            "AH" | "BH" | "CH" | "DH" => Ok((value & 0x0000FF00) >> 8),
            "FLAG" | "AX" | "BX" | "CX" | "DX"  => Ok(value & 0x0000FFFF), 
            "ESI" | "EDI" | "IP" | "EAX" | "EBX" | "ECX" | "EDX"  => Ok(value), 
            _ => Err(ErrorCode::InvalidRegister)
        }
    }
    
    fn set_flags(&mut self, result: usize, size: VariableSize, overflowed: bool) {

        self.set_register_value("FLAG", 0).expect("Flag Register Missing?!");

        if result.count_ones() % 2 == 0 {
            self.set_flag(Flag::Parity, true);

        }

        // Set Zero Flag
        if result == 0 {
            self.set_flag(Flag::Zero, true);    
        }

        // Set Sign Flag
        let mask = (0xFF as usize) << (size.value() - 1) * 8;
        let displacement = size.value() * 8 - 1;
        if (result & mask) >> displacement == 1 {
            self.set_flag(Flag::Sign, true);    
        }

        // Set Overflow Flag
        if overflowed {
            self.set_flag(Flag::Overflow, true);    
        }
        
    }

    fn save_label(&mut self, name: String) -> Result<(), ErrorCode> {

        if self.labels.insert(name, self.get_register_value("IP")? as usize).is_some() {
            return Err(ErrorCode::VariableAlreadyExists);
        } else {
            Ok(())
        }
    }

    fn get_pointer_argument_size<'a>(&self, argument: &'a str) -> (Option<VariableSize>, &'a str) {
        let parts: Vec<&str> = argument.split_whitespace().collect();
        match parts.as_slice() {
            ["BYTE", "PTR", arg] => (Some(VariableSize::Byte), *arg),
            ["WORD", "PTR", arg] => (Some(VariableSize::Word), *arg),
            ["DWORD", "PTR", arg] => (Some(VariableSize::DoubleWord), *arg),
            [arg] => {

                if let Ok((_, size)) = self.parse_value_from_parameter(*arg, None) {
                    (Some(size), *arg)
                } else {
                    (None, *arg)
                }
            }
            _ => (None, argument)
        }
    }

    pub fn parse_value_from_parameter(&self, parameter: &str, mut size: Option<VariableSize>) -> Result<(u32, VariableSize), ErrorCode> {
        if let Some(value) = parse_string_to_usize(parameter){
            // Constant
            let assumed_size = if value <= u8::MAX.into()  {
                VariableSize::Byte
            } else if value > u8::MAX.into() && value <= u16::MAX.into() {
                VariableSize::Word
            }  else if value > u16::MAX.into() && value <= u32::MAX as usize {
                VariableSize::DoubleWord
            } else {
                return Err(ErrorCode::InvalidValue(format!("value {value} is outside of allowed range. 0-u32::MAX")));
            };

            Ok((value as u32, assumed_size))

        } else if let Ok(value) = self.get_register_value(parameter) {
            // Register
            let assumed_size = match parameter {
                "AL"|"AH"|"BL"|"BH"|"CL"|"CH"|"DL"|"DH" => {
                    VariableSize::Byte
                },
                "EAX"|"EAB"|"EAC"|"EAD"|"ESI"|"EDI" => {
                    VariableSize::DoubleWord
                },
                "AX"|"BX"|"CX"|"DX"|"SI"|"DI"|"FLAG"|"IP"  => {
                    VariableSize::Word
                },
                _ => return Err(ErrorCode::InvalidRegister) // Unreachable
            };
            Ok((value, assumed_size))
        } else if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(parameter, &self.registers, &self.labels, true) {
   
            // Memory (adjust size to pointer size)
            let memory_address_str = parameter.to_string();
            if memory_address_str.len() < 3 {
                return Err(ErrorCode::InvalidPointer("Pointer length must be more than 3 including []".to_string()));
            }
            if size.is_none() {
                if let Some(metadata) = self.memory_manager.get_variable(&memory_address_str[1..memory_address_str.len()-1]) {
                    size = Some(metadata.size);
                } else if let Some(_) = self.labels.get(&memory_address_str[1..memory_address_str.len()-1]) {
                    size = Some(VariableSize::Word);
                }
                
            }

            match size {
                Some(VariableSize::Byte)|None => {
                    let a = self.memory_manager.get_byte(parsed_address)?;
                    Ok((a as u32, VariableSize::Byte))
                },
                Some(VariableSize::Word) => {
                    let a = self.memory_manager.get_word(parsed_address)?;
                    Ok((a as u32, VariableSize::Word))
                },
                Some(VariableSize::DoubleWord) => {
                    let a = self.memory_manager.get_dword(parsed_address)?;
                    Ok((a as u32, VariableSize::DoubleWord))
                },
            }
        } else {
            Err(ErrorCode::InvalidValue(format!("Parameter {parameter} could not be parsed.")))
        }
    }

    pub fn execute(&mut self, verbose: bool) -> Result<(), ErrorCode>{
        // Go over code to get all the labels.
        loop {
            let maybe_arguments = self.lines.next(&mut self.registers[get_register("IP")], false);

            if maybe_arguments.is_none() {
                break;
            }
            let str_maybe_arguments = maybe_arguments.unwrap();
            let arguments: Vec<&str> = convert_vec_string_to_vec_str(&str_maybe_arguments);

            // Process the instruction based on the arguments
            match arguments.as_slice() {
                [label] if label.ends_with(":") && self.memory_manager.is_valid_variable_name(&label[..label.len()-1]) => {
                    self.save_label(label[..label.len()-1].to_string())?;
                },
                _ => {},
            }
        }

        self.lines.set_ip(0);

        loop {
            // Check if the verbose flag is set
            
            let maybe_arguments = self.lines.next(&mut self.registers[get_register("IP")], verbose);

            if maybe_arguments.is_none() {
                break;
            }

            let str_maybe_arguments = maybe_arguments.unwrap();
            let arguments: Vec<&str> = convert_vec_string_to_vec_str(&str_maybe_arguments);

            let ip = self.registers[get_register("IP")].get_word();
            // Process the instruction based on the arguments
            match arguments.as_slice() {
                    // size @ ("WORD" | "BYTE" | "DWORD"), "PTR", 
                    [op @ ("inc" | "dec"), register] if self.is_valid_register(register) => {
                    let inc = *op == "inc";
                    let (result, overflowed) = match get_register_size(register).unwrap() {
                        VariableSize::Byte => {
                            let (value, overflowing) = if inc {
                                (self.get_register_value(register)? as u8).overflowing_add(1)

                            } else {
                                (self.get_register_value(register)? as u8).overflowing_sub(1)
                            };

                            self.set_register_value(register, value as u32)?;
                            (value as u32, overflowing)
                        },
                        VariableSize::Word => {
                            let (value, overflowing) = if inc {
                                (self.get_register_value(register)? as u16).overflowing_add(1)

                            } else {
                                (self.get_register_value(register)? as u16).overflowing_sub(1)
                            };

                            self.set_register_value(register, value as u32)?;
                            (value as u32, overflowing)
                        },
                        VariableSize::DoubleWord => {
                            let (value, overflowing) = if inc {
                                (self.get_register_value(register)? as u32).overflowing_add(1)

                            } else {
                                (self.get_register_value(register)? as u32).overflowing_sub(1)
                            };

                            self.set_register_value(register, value)?;
                            (value, overflowing)
                        },

                    };
                    self.set_flags(result as usize, get_register_size(register).unwrap(), overflowed)
                },
                [op @ ("inc" | "dec"), memory_address] if self.memory_manager.is_memory_operand(memory_address) => {
                                    // Case with WORD/BYTE PTR

                    let (size_option, memory_address_str) = self.get_pointer_argument_size(memory_address);
                    let inc = *op == "inc";
                    let size = size_option.unwrap_or(VariableSize::Byte);
                    // Calculate effective address
                    match self.memory_manager.calculate_effective_address(memory_address_str, &self.registers, &self.labels, true) {
                        Ok(parsed_address) => {
                            let (current, overflowed) = match size {
                                VariableSize::Byte => {
                                    if inc {
                                        let (cu8, o) = self.memory_manager.get_byte(parsed_address)?.overflowing_add(1);
                                        (cu8 as u32, o)
                                    } else {
                                        let (cu8, o) = self.memory_manager.get_byte(parsed_address)?.overflowing_sub(1);
                                        (cu8 as u32, o)
                                    }
                                },
                                VariableSize::Word => {
                                    if inc {
                                        let (cu16, o) = self.memory_manager.get_word(parsed_address)?.overflowing_add(1);
                                        (cu16 as u32, o)
                                    } else {
                                        let (cu16, o) = self.memory_manager.get_word(parsed_address)?.overflowing_sub(1);
                                        (cu16 as u32, o)
                                    }
                                },
                                VariableSize::DoubleWord => {
                                    if inc {
                                        let (cu16, o) = self.memory_manager.get_dword(parsed_address)?.overflowing_add(1);
                                        (cu16 as u32, o)
                                    } else {
                                        let (cu16, o) = self.memory_manager.get_dword(parsed_address)?.overflowing_sub(1);
                                        (cu16 as u32, o)
                                    }
                                }
                            };
                            
                            self.set_flags(current as usize, size, overflowed);
                            
                            match size {
                                VariableSize::Byte => self.memory_manager.set_byte(parsed_address, current as u8)?,
                                VariableSize::Word => self.memory_manager.set_word(parsed_address, current as u16)?,
                                VariableSize::DoubleWord => self.memory_manager.set_dword(parsed_address, current as u32)?,
                            };
                        },
                        Err(error) => return Err(error),
                    }
                },

                ["inc", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Inc));
                    
                    return Err(ErrorCode::InvalidOpcode);
                },
                ["dec", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Dec));
                    
                    return Err(ErrorCode::InvalidOpcode);
                },

                ["lea", reg, memory_address] if self.is_valid_register(*reg) && self.memory_manager.is_memory_operand(memory_address) => {
                    match self.memory_manager.calculate_effective_address(memory_address, &self.registers, &self.labels, true) {
                        Ok(parsed_address) => {
                            // Determine the register size
                            match get_register_size(*reg) {
                                Some(VariableSize::Byte) => {
                                    return Err(ErrorCode::NotEnoughSpace(
                                        format!("Cannot store pointer in 1-byte register: {reg}."))
                                    );
                                },
                                Some(VariableSize::Word)|Some(VariableSize::DoubleWord) => {
                                    self.mov_reg_const(reg, parsed_address as u32)?;
                                },
                                _ => {
                                    return Err(ErrorCode::InvalidRegister);
                                },
                            }
                        },
                        Err(error) => return Err(error),
                    }
                },
                ["lea", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Lea));
                    
                    return Err(ErrorCode::InvalidOpcode);
                },
                // MOV Instructions
                // OP    REG      MEM/REG/CONST
                ["mov", reg, parameter] if self.is_valid_register(*reg) => {
                    let (size_option, memory_address) = self.get_pointer_argument_size(parameter);
                    let (constant, assumed_size) = self.parse_value_from_parameter(memory_address, size_option)?;
                    if let Some(reg_size) = get_register_size(reg) {
                        if assumed_size.value() > reg_size.value() {
                            return Err(ErrorCode::InvalidValue(format!("Value {parameter} cannot fit in {reg}")));
                        }
                    }
                    self.mov_reg_const(reg, constant)?;

                },
                // OP       MEM               REG/CONST
                ["mov", memory_address, parameter] => {

                    if self.memory_manager.is_memory_operand(parameter) {
                        return Err(ErrorCode::InvalidValue(format!("Direct memory transfer is not supported.")));
                    }

                    let (size_option_dst, _) = self.get_pointer_argument_size(parameter);


                    let (size_option_src, sliced_memory_string) = self.get_pointer_argument_size(memory_address);

                    if let Some(size_dst) = size_option_dst {
                        if let Some(size_src) = size_option_src {
                            if size_dst != size_src {
                                return Err(ErrorCode::InvalidPointer(format!("Source ({parameter}) and destination ({memory_address}) size don't match.")));
                            }
                        }
                    };
                    

                    // Calculate effective address of destination            
                    match self.memory_manager.calculate_effective_address(sliced_memory_string, &self.registers, &self.labels, true) {
                        // Destination is valid address
                        Ok(parsed_address) => {
                            // Get value and size of memory to mov into destination address
                            let (constant, assumed_size_parameter ) = {
                                let (v, s) = self.parse_value_from_parameter(parameter, size_option_src)?;
                                (v, size_option_src.unwrap_or(s))
                            };
                            let (_, assumed_size_memory) = self.parse_value_from_parameter(sliced_memory_string, size_option_src)?; 
                            if assumed_size_parameter != assumed_size_memory {
                                return Err(
                                    ErrorCode::InvalidValue(
                                        format!("Target memory pointer size ({}) doesn't match second parameter size ({})",
                                                         assumed_size_memory.value(),          assumed_size_parameter.value())
                                    )
                                );
                            }
                            
                            match assumed_size_parameter {
                                VariableSize::Byte => self.memory_manager.set_byte(parsed_address, constant as u8)?,
                                VariableSize::Word => self.memory_manager.set_word(parsed_address, constant as u16)?,
                                VariableSize::DoubleWord => self.memory_manager.set_dword(parsed_address, constant as u32)?,
                            };
                        },
                        // Destination is not valid address
                        Err(error) => return Err(error)
                    }
                },  
                // HELP COMMAND
                ["mov", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Mov));
                    
                    return Err(ErrorCode::InvalidOpcode);
                },

                // ADD/SUB Instructions
                //         OP              REG          MEM/REG/CONST
                [op @ ("add"|"sub"), reg, parameter] if self.is_valid_register(reg) => {
                    let is_addition = *op == "add";
                                                                // No "WORD PTR" etc.
                    let (size_option, trimmed_parameter) = self.get_pointer_argument_size(parameter);
    
                    let (constant, assumed_size)= self.parse_value_from_parameter(trimmed_parameter, size_option)?;

                    if let Some(reg_size) = get_register_size(reg) {
                        if assumed_size.value() != reg_size.value() {
                            return Err(ErrorCode::InvalidValue(format!("Value {parameter} cannot fit in {reg}")));
                        }
                    }
                    self.add_or_sub_reg_const(reg, constant, is_addition)?;
                },

                //         OP               MEM          REG/CONST
                [op @ ("add"|"sub"), memory_address, parameter] if self.memory_manager.is_memory_operand(memory_address) => {
                    let is_addition = *op == "add";

                    if self.memory_manager.is_memory_operand(parameter) {
                        return Err(ErrorCode::InvalidValue(format!("Direct memory transfer is not supported.")));
                    }
                    
                    let (size_option_src, _) = self.get_pointer_argument_size(parameter);
                    let (size_option_dest, memory_address_str_dest) = self.get_pointer_argument_size(memory_address);

                    if size_option_src != size_option_dest {
                        return Err(ErrorCode::InvalidPointer("Source and destination size don't match.".to_string()))
                    }

                    // Calculate effective address of destination            
                    match self.memory_manager.calculate_effective_address(memory_address_str_dest, &self.registers, &self.labels, true) {
                        // Destination is valid address
                        Ok(parsed_address) => {
                            // Get value and size of memory to mov into destination address
                            let (constant, assumed_size ) = {
                                self.parse_value_from_parameter(parameter, size_option_src)?
                            };

                            // Save EAX value
                            let eax: u32 = self.get_register_value("EAX")?;
                            // Load constant into EAX
                            self.registers[get_register("EAX")].load_dword(constant);
                            match assumed_size {
                                VariableSize::Byte => self.add_or_sub_mem_reg(parsed_address, "AL", is_addition)?,
                                VariableSize::Word =>  self.add_or_sub_mem_reg(parsed_address, "AX", is_addition)?,
                                VariableSize::DoubleWord =>  self.add_or_sub_mem_reg(parsed_address, "EAX", is_addition)?,
                            };
                            // Load original value back.
                            self.registers[get_register("EAX")].load_dword(eax);


                        },
                        // Destination is not valid address
                        Err(error) => return Err(error)
                    }

                    
                    
                },

                ["add", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Add));
                    
                    return Err(ErrorCode::InvalidOpcode);
                },

                ["sub", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Sub));
                    
                    return Err(ErrorCode::InvalidOpcode);
                },

                [op @ ("mul" | "imul"), parameter] => {
                    // Determine size of the operand
                    let (size_option_src, memory_address_str_src) = self.get_pointer_argument_size(parameter);

                    let (src_value, backup_size) = self.parse_value_from_parameter(memory_address_str_src, None)?;

                    let size = size_option_src.unwrap_or(backup_size);

                    self.mul_value(src_value, size, *op == "imul")?;
                },

                [op @ ("mul" | "imul"), _rest @ ..] => {
                    let signed = *op == "imul";
                    if signed {
                        println!("{}", Command::get_help_string(Command::Imul));
                    } else {
                        println!("{}", Command::get_help_string(Command::Mul));
                    }
                    
                    return Err(ErrorCode::InvalidOpcode);
                },


                // // DIV Instructions

                [op @ ("div" | "idiv"), parameter] => {
                    // Determine size of the operand
                    let (size_option_src, memory_address_str_src) = self.get_pointer_argument_size(parameter);

                    let (src_value, backup_size) = self.parse_value_from_parameter(memory_address_str_src, None)?;

                    let size = size_option_src.unwrap_or(backup_size);

                    self.div_value(src_value, size, *op == "imul")?;
                },
            

                [op @ ("div" | "idiv"), _rest @ ..] => {
                    let signed = *op == "idiv";
                    if signed {
                        println!("{}", Command::get_help_string(Command::Idiv));
                    } else {
                        println!("{}", Command::get_help_string(Command::Div));
                    }
                    
                    return Err(ErrorCode::InvalidOpcode);
                },

                // // PRINT  Instructions

                ["print", parameter] => { // if self.memory_manager.is_memory_operand(memory_address) => {
                    // Determine size of the operand

                    let (size, memory_address_str_src) = self.get_pointer_argument_size(parameter);
                    let (src_value, _) = self.parse_value_from_parameter(memory_address_str_src, size)?;

                    println!("[PRINT]@[IP={ip}] {parameter}: {0}\n", src_value);

                },


                ["print", number, memory_address]  => { //if self.memory_manager.is_memory_operand(memory_address) &&
                                                                                            //parse_string_to_usize(*number).is_some() => {

                    let (size_option_src, trimmed_address) = self.get_pointer_argument_size(memory_address);
                    let size = size_option_src.unwrap_or(VariableSize::Byte);
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(trimmed_address, &self.registers, &self.labels, true) {
                        if let Some(value) = parse_string_to_usize(*number) {

                            self.memory_manager.check_memory_address(parsed_address+value*size.value())?;

                            let ip = self.registers[get_register("IP")].get_word();
                            print!("[PRINT]@[IP={ip}] [{parsed_address}..{}]: [ ", parsed_address+value-1);
                            for i in 0..value {
                                match size {
                                    VariableSize::Byte => {
                                        print!("{0} ", self.memory_manager.get_byte(parsed_address+i*size.value())?);
                                    },
                                    VariableSize::Word => {
                                        print!("{0} ", self.memory_manager.get_word(parsed_address+i*size.value())?);
                                    },
                                    VariableSize::DoubleWord => {
                                        print!("{0} ", self.memory_manager.get_dword(parsed_address+i*size.value())?);
                                    },

                                }
                            }
                            println!("]");
                        } else {
                            return Err(ErrorCode::InvalidValue(format!("Integer {number} could not be parsed.")))
                        }
                    } else {
                        return Err(ErrorCode::InvalidValue(format!("Memory address {memory_address} could not be parsed.")))
                    }
                },
                // ["int", interrupt] => if parse_string_to_usize(*interrupt) == Some(0){

                // }
                ["NOP"] => {},
                [variable_name, define_as @ ("db"|"dw"|"dd"), rest @ ..] if self.memory_manager.is_valid_variable_name(*variable_name) => {
                // (parse_string_to_usize(*data).is_some() || self.memory_manager.is_valid_array(*data).is_ok() )=> {
                    let size: VariableSize = match *define_as {
                        "db" => VariableSize::Byte,
                        "dw" => VariableSize::Word,
                        "dd" => VariableSize::DoubleWord,
                        _ => return Err(ErrorCode::InvalidValue("Invalid Variable Size".to_string()))
                    };
                    let mut bytes: Vec<u32> = Vec::new();

                    for (_, &arg) in rest.iter().enumerate() {
                        // Check if argument is a string literal and remove surrounding quotes
                        if let Some((start_char, end_char)) = arg.chars().next().zip(arg.chars().rev().next()) {
                            if (start_char == '"' && end_char == '"') || (start_char == '\'' && end_char == '\'') {
                                let inner = &arg[1..arg.len() - 1];
                                for c in inner.chars() {
                                    bytes.push(c as u32);
                                }
                            } else {
                                // Handle other cases (numeric values, etc.)
                                if let Some(value) = parse_string_to_usize(arg) {
                                    bytes.push(value as u32);
                                } else {
                                    return Err(ErrorCode::InvalidValue(format!("Could not parse {arg}")));
                                }
                            }
                        } else {
                            // Handle other cases (numeric values, etc.)
                            if let Some(value) = parse_string_to_usize(arg) {
                                bytes.push(value as u32);                   
                            } else {
                                return Err(ErrorCode::InvalidValue(format!("Could not parse {arg}")));
                            }
                        }
                    }

                    if let Err(error) = self.memory_manager.save_variable(variable_name.to_string(), &bytes, size) {
                        
                        return Err(error);
                    }

                },

                //////// JUMPS ////////////
                ["jmp", label] => {
                    if let Err(error) = self.jump_to(label) {     
                        println!("{}", Command::get_help_string(Command::Jmp));
                        return Err(error);
                    }
                }

                [_flag @ ("je"|"jz"|"jne"|"jnz"), label] => {
                    let ne = *_flag == "jne" || *_flag == "jnz";
                    if !(ne ^ self.is_flag_on(Flag::Zero)) {
                        continue;
                    }
                    if let Err(error) = self.jump_to(label) {     
                        match *_flag {
                            "je" =>  println!("{}", Command::get_help_string(Command::Je)),
                            "jz" =>  println!("{}", Command::get_help_string(Command::Jz)),
                            "jne" =>  println!("{}", Command::get_help_string(Command::Jne)),
                            "jnz" =>  println!("{}", Command::get_help_string(Command::Jnz)),
                            _ => {}
                        }          
                        return Err(error);
                    }
                }
                
                [_flag @ ("jg" | "jge"), label] => {
                    let is_jg = *_flag == "jg";

                    if !(is_jg && (self.is_flag_on(Flag::Sign) ^ self.is_flag_on(Flag::Overflow))) {
                        continue;
                    }
            
                    if let Err(error) = self.jump_to(label) {   
                        match *_flag {
                            "jg" =>  println!("{}", Command::get_help_string(Command::Jg)),
                            "jge" =>  println!("{}", Command::get_help_string(Command::Jge)),
                            _ => {}
                        }                
                        return Err(error);
                    }
                }

                [_flag @ ("jl" | "jle"), label] => {
                    let is_jl = *_flag == "jl";

                    if !(is_jl && (self.is_flag_on(Flag::Sign) ^ self.is_flag_on(Flag::Overflow))) {
                        continue;
                    }
            
                    if let Err(error) = self.jump_to(label) {    
                        match *_flag {
                            "jl" =>  println!("{}", Command::get_help_string(Command::Jl)),
                            "jle" =>  println!("{}", Command::get_help_string(Command::Jle)),
                            _ => {}
                        }   
                        return Err(error);
                    }
                }

                [_flag @ ("ja" | "jae"), label] => {
                    let is_jl = *_flag == "ja";

                    if !(is_jl && (self.is_flag_on(Flag::Carry) ^ self.is_flag_on(Flag::Zero))) {
                        continue;
                    }
            
                    if let Err(error) = self.jump_to(label) {       
                        match *_flag {
                            "ja" =>  println!("{}", Command::get_help_string(Command::Ja)),
                            "jae" =>  println!("{}", Command::get_help_string(Command::Jae)),
                            _ => {}
                        }           
                        return Err(error);
                    }
                }

                [_flag @ ("jb" | "jbe"), label] => {
                    let is_jbe = *_flag == "jbe";

                    if self.is_flag_on(Flag::Carry) || (is_jbe && self.is_flag_on(Flag::Zero)) {
                        continue;
                    }

                    if let Err(error) = self.jump_to(label) {          
                        match *_flag {
                            "jb" =>  println!("{}", Command::get_help_string(Command::Jb)),
                            "jbe" =>  println!("{}", Command::get_help_string(Command::Jbe)),
                            _ => {}
                        }        
                        return Err(error);
                    }
                }

                
                ["cmp", first_operand, second_operand] => {
                        // Determine if operands are negative (assuming they start with '-')
                        let is_negative_first = first_operand.starts_with("-");
                        let is_negative_second = second_operand.starts_with("-");
                    
                        let arg1 = if is_negative_first {
                            first_operand[1..].to_string()
                        } else {
                            first_operand.to_string()
                        };
                        
                        let arg2 = if is_negative_second {
                            second_operand[1..].to_string()
                        } else {
                            second_operand.to_string()
                        };
                    
                        // Initialize variables to store parsed values
                        let first_value: isize;
                        let second_value: isize;
                    
                        // Process first operand
                        if self.memory_manager.is_memory_operand(&arg1) {
                            match self.memory_manager.calculate_effective_address(&arg1, &self.registers, &self.labels, false) {
                                Ok(value) => first_value = value as isize,
                                Err(error) => return Err(error),
                            }
                        } else if let Some(v) = self.memory_manager.parse_value(&arg1, is_negative_first, &self.registers, &self.labels, false) {
                            first_value = v as isize;
                        } else {
                            return Err(ErrorCode::InvalidValue(format!("Could not parse {}", first_operand)));
                        }
                    
                        // Process second operand
                        if self.memory_manager.is_memory_operand(&arg2) {
                            match self.memory_manager.calculate_effective_address(&arg2, &self.registers, &self.labels, false) {
                                Ok(value) => second_value = value as isize,
                                Err(error) => return Err(error),
                            }
                        } else if let Some(v) = self.memory_manager.parse_value(&arg2, is_negative_second, &self.registers, &self.labels, false) {
                            second_value = v as isize;
                        } else {
                            return Err(ErrorCode::InvalidValue(format!("Could not parse {}", second_operand)));
                        }
                    
                        // Set flags based on comparison results
                        self.set_flag(Flag::Zero, first_value == second_value);
                        self.set_flag(Flag::Carry, first_value < second_value);
                        self.set_flag(Flag::Overflow, first_value > second_value);
                        self.set_flag(Flag::Sign, first_value - second_value < 0);

                }
                ["push", parameter] => {
                    // No "WORD PTR" etc.

                    let (size_option, trimmed_parameter) = self.get_pointer_argument_size(parameter);

                    let (value, size) = self.parse_value_from_parameter(trimmed_parameter, size_option)?;
                    self.memory_manager.push_to_stack(value, size)?;

                },
                ["pop", parameter] => {
                    // No "WORD PTR" etc.
                    let (size_option, trimmed_parameter) = self.get_pointer_argument_size(parameter);

                    let (_, size) = self.parse_value_from_parameter(trimmed_parameter, size_option)?;
                    
                    let popped_value = self.memory_manager.pop_from_stack(size)?;

                    if self.is_valid_register(parameter) {
                        match size {
                            VariableSize::Byte => return Err(ErrorCode::InvalidValue("POP can only receive a 16-bit or 32-bit parameter.".to_string())),
                            VariableSize::Word => self.registers[get_register(parameter)].load_word(popped_value as u16),
                            VariableSize::DoubleWord => self.registers[get_register(parameter)].load_dword(popped_value),
                        }
                    } else {
                        // Calculate effective address of destination            
                        match self.memory_manager.calculate_effective_address(trimmed_parameter, &self.registers, &self.labels, true) {
                            // Destination is valid address
                            Ok(parsed_address) => {
                                match size {
                                    VariableSize::Byte =>  return Err(ErrorCode::InvalidValue("POP can only receive a 16-bit or 32-bit parameter.".to_string())),
                                    VariableSize::Word => self.memory_manager.set_word(parsed_address, popped_value as u16)?,
                                    VariableSize::DoubleWord => self.memory_manager.set_dword(parsed_address, popped_value)?,
                                }
                            },
                            Err(error) => return Err(error)
                        }
                    }

                },
                [label] if label.ends_with(":") => {
                    let no_colon = &label[..label.len()-1];
                    // Found no label
                    if self.labels.get(no_colon).is_none(){
                        return Err(ErrorCode::InvalidOpcode);
                    }
                }
                _ => {
                    println!("Unknown instruction: {:?}", arguments);
                    return Err(ErrorCode::InvalidOpcode);
                    // Handle unrecognized instructions
                },
            }
        }
        
        Ok(())
    }

    fn jump_to(&mut self, label: &&str) -> Result<(), ErrorCode> {
        if let Some(address) = self.labels.get(*label) {
            self.lines.set_ip(*address);
        } else {
            let (size_option, trimmed) = self.get_pointer_argument_size(label);

            if trimmed != *label {
                return Err(ErrorCode::InvalidPointer(
                    format!("Invalid Syntax.")
                ));
            }
            
            let target =  if self.memory_manager.is_memory_operand(label) {
                self.memory_manager.calculate_effective_address(label, &self.registers, &self.labels, true)?
                
            } else {
                let (a, _) = self.parse_value_from_parameter(label, size_option)?;
                a as usize
            };

            if target > MEMORY_SIZE - 1 {
                return Err(ErrorCode::InvalidPointer(
                    format!("Target IP \"{}\" is outside of memory bounds.", target)
                ));
            }

            self.lines.set_ip(target as usize);

        }
        Ok(())
    }

    fn is_flag_on(&self, flag: Flag) -> bool {
        // No need to check, because we know FLAG is a valid register.
        // Also, it's 16 bits, i.e., we can convert it without issues.
        let value = self.get_register_value("FLAG").unwrap() as u16;
        value & flag.value() != 0
    }


    // MOV operations
    // REG <- Reg, Const, Var, Mem
    // Mem <- Reg, Const, Var, Mem
    // Var <- Reg, Const, Var, Mem
    // Const <- NOTHING, Const can't be moved into

    fn mov_reg_const(&mut self, dest: &str, constant: u32) -> Result<(), ErrorCode>{
        let size = match dest {
            reg if reg.ends_with('L')|reg.ends_with('H') => {
                self.registers[get_register(dest)].load_byte(constant as u8, reg.ends_with('H'));
                VariableSize::Byte
            },
            "AX"|"BX"|"CX"|"DX"|"SI"|"DI"|"FLAG"|"IP" => {
                self.registers[get_register(dest)].load_word(constant as u16);
                VariableSize::Word
            },
            reg if reg.starts_with("E") => {
                self.registers[get_register(dest)].load_dword(constant);
                VariableSize::DoubleWord
            }
            _ => return Err(ErrorCode::InvalidRegister)
        };
        self.set_flags(constant as usize, size, false);
        Ok(())
    }

    fn add_or_sub_mem_reg(&mut self, memory_address: usize, src: &str, is_addition: bool) -> Result<(), ErrorCode> {

        let (result, overflowed, size) = match src {
            reg if reg.ends_with('L') || reg.ends_with('H') => {
                let src_value = self.registers[get_register(src)].get_byte(reg.ends_with('H'));
                let dest_value = self.memory_manager.get_byte(memory_address)?;

                let (sum, overflowed) = if is_addition {
                    dest_value.overflowing_add(src_value)
                } else {
                    dest_value.overflowing_sub(src_value)
                };

                self.memory_manager.set_byte(memory_address, sum)?;


                (sum as u32, overflowed, VariableSize::Byte)
            },
            "AX" | "BX" | "CX" | "DX" | "SI" | "DI" | "FLAG" | "IP" => {
                let src_value = self.registers[get_register(src)].get_word();
                let dest_value = self.memory_manager.get_word(memory_address)?;

                let (sum, overflowed) = if is_addition {
                    dest_value.overflowing_add(src_value)
                } else {
                    dest_value.overflowing_sub(src_value)
                };

                self.memory_manager.set_word(memory_address, sum)?;

                (sum as u32, overflowed, VariableSize::Word)
            },
            reg if reg.starts_with('E') => {
                self.memory_manager.check_memory_address(memory_address)?;

                let src_value = self.registers[get_register(src)].get_dword();
                let dest_value = self.memory_manager.get_dword(memory_address)?;

                let (sum, overflowed) = if is_addition {
                    dest_value.overflowing_add(src_value)
                } else {
                    dest_value.overflowing_sub(src_value)
                };

                self.memory_manager.set_dword(memory_address, sum)?;

                (sum as u32, overflowed, VariableSize::DoubleWord)
            },
            _ => return Err(ErrorCode::InvalidRegister),
        };
        self.set_flags(result as usize, size, overflowed);
        Ok(())
    }
    
    fn add_or_sub_reg_const(&mut self, dest: &str, constant: u32, is_addition: bool) -> Result<(), ErrorCode>{
        match dest {
            "AX"|"BX"|"CX"|"DX"|"ESI"|"EDI"|"FLAG"|"IP" => {
                if constant > u16::MAX as u32 {
                    return Err(ErrorCode::InvalidValue(format!("Value {constant} can't fit in {dest}")));
                }
                let dest_value = self.get_register_value(dest)? as u16;
                let (sum, overflowed) = if is_addition {
                    dest_value.overflowing_add(constant as u16)
                } else {
                    dest_value.overflowing_sub(constant as u16)
                };                
                self.registers[get_register(dest)].load_word(sum);
                self.set_flags(sum as usize,VariableSize::Byte,  overflowed);
            }
            reg if reg.ends_with('L') || reg.ends_with('H') => {
                if constant > u8::MAX as u32 {
                    return Err(ErrorCode::InvalidValue(format!("Value {constant} can't fit in {dest}")));
                }
                let dest_value = self.get_register_value(dest)? as u8;
                let (sum, overflowed) = if is_addition {
                    dest_value.overflowing_add(constant as u8)
                } else {
                    dest_value.overflowing_sub(constant as u8)
                };                
                self.registers[get_register(dest)].load_byte(sum,  reg.ends_with('H'));
                self.set_flags(sum as usize,VariableSize::Byte,  overflowed);
            },
            reg if reg.starts_with('E') => {
                let dest_value = self.get_register_value(dest)? as u32;

                let (sum, overflowed) = if is_addition {
                    dest_value.overflowing_add(constant)
                } else {
                    dest_value.overflowing_sub(constant)
                };          
                self.registers[get_register(dest)].load_dword(sum);
                
                self.set_flags(sum as usize,VariableSize::DoubleWord, overflowed);
            },
            _ => {
                return Err(ErrorCode::InvalidRegister);
            }
        };
        Ok(())
    }

    fn set_register_value(&mut self, register: &str, value: u32) -> Result<(), ErrorCode>{
        match get_register_size(register) {
            Some(VariableSize::Byte) => {
                if value > u8::MAX as u32 {
                    return Err(ErrorCode::InvalidValue(format!("Value {value} can't fit in {register}")));
                }
                self.registers[get_register(register)].load_byte(value.try_into().unwrap(), register.ends_with('H'));
                Ok(())
            },
            Some(VariableSize::Word) => {
                if value > u16::MAX as u32 {
                    return Err(ErrorCode::InvalidValue(format!("Value {value} can't fit in {register}")));
                }
                self.registers[get_register(register)].load_word(value.try_into().unwrap());
                Ok(())
            }
            Some(VariableSize::DoubleWord) => {
                self.registers[get_register(register)].load_dword(value.try_into().unwrap());
                Ok(())
            }
            _ => Err(ErrorCode::InvalidRegister)
        }
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            let current = self.registers[get_register("FLAG")].get_word();
            self.registers[get_register("FLAG")].load_word(current | flag.value());
        } else {
            let current = self.registers[get_register("FLAG")].get_word();
            self.registers[get_register("FLAG")].load_word(current & !flag.value());
        }
    }

    //////////// MUL ////////////
    // Multiply the value in the source register by the value in AX register.

    fn mul_value(&mut self, src_value: u32, size: VariableSize , signed: bool) -> Result<(), ErrorCode>{
        match size {
            VariableSize::Byte => self.mul_8bit(src_value as u8, signed),
            VariableSize::Word => self.mul_16bit(src_value as u16, signed),
            VariableSize::DoubleWord => self.mul_32bit(src_value, signed),
        }
    }
    
    // Function to multiply 8-bit values and store the result in AX register.
    fn mul_8bit(&mut self, src_value: u8, signed: bool) -> Result<(), ErrorCode>{
        let ax_value = self.get_register_value("AX")?;
        let al_value = ax_value as u8;
    
        let result = if signed {
            let al_signed = al_value as i8;
            let src_signed = src_value as i8;
            (al_signed as i16 * src_signed as i16) as u16
        } else {
            (al_value as u16 * src_value as u16) as u16
        };
    
        let overflow_condition = result > 0xFF;
    
        self.set_flag(Flag::Carry, overflow_condition);
        self.set_flag(Flag::Overflow, overflow_condition);
        self.set_register_value("AX", result as u32)?;
        Ok(())
    }
    
    // Function to multiply 16-bit values and store the result in DX:AX register.
    fn mul_16bit(&mut self, src_value: u16, signed: bool) -> Result<(), ErrorCode> {
        let ax_value = self.get_register_value("AX")?;
    
        let result = if signed {
            let ax_signed = ax_value as i16;
            let src_signed = src_value as i16;
            (ax_signed as i32 * src_signed as i32) as u32
        } else {
            (ax_value as u32 * src_value as u32) as u32
        };
    
        let overflow_condition = result > 0xFFFF;
    
        self.set_flag(Flag::Carry, overflow_condition);
        self.set_flag(Flag::Overflow, overflow_condition);
        self.set_register_value("AX", result as u32)?;
        self.set_register_value("DX", (result >> 16) as u32)?;
        Ok(())
    }
    
    // Function to multiply 32-bit values and store the result in EDX:EAX register.
    fn mul_32bit(&mut self, src_value: u32, signed: bool)-> Result<(), ErrorCode> {
        let eax_value = self.get_register_value("EAX")?;
    
        let result = if signed {
            let eax_signed = eax_value as i32;
            let src_signed = src_value as i32;
            (eax_signed as i64 * src_signed as i64) as u64
        } else {
            (eax_value as u64 * src_value as u64) as u64
        };
    
        let overflow_condition = result > 0xFFFFFFFF;
    
        self.set_flag(Flag::Carry, overflow_condition);
        self.set_flag(Flag::Overflow, overflow_condition);
        self.set_register_value("EAX", result as u32)?;
        self.set_register_value("EDX", (result >> 32) as u32)?;
        Ok(())
    }



    /////////// DIV ////////////
    // Divide the value in the source register by the value in the AX register.
    fn div_value(&mut self, src_value: u32, size: VariableSize , signed: bool) -> Result<(), ErrorCode>{
        match size {
            VariableSize::Byte => self.div_8bit(src_value as u8, signed),
            VariableSize::Word => self.div_16bit(src_value as u16, signed),
            VariableSize::DoubleWord => self.div_32bit(src_value, signed),
        }
    }
    // AX / src_value, AH = AX % src_value
    fn div_8bit(&mut self, src_value: u8, signed: bool) -> Result<(), ErrorCode> {
        let ax_value = self.get_register_value("AX")? as u16;
        if src_value == 0 {
            return Err(ErrorCode::DivisionByZero);
        }

        let (quotient, remainder) = if signed {
            let al_signed = (ax_value as u8) as i8;
            let src_signed = src_value as i8;
            let (q, r) = (al_signed / src_signed, al_signed % src_signed);
            (q as u8, r as u8)
        } else {
            ((ax_value / src_value as u16) as u8, (ax_value % src_value as u16) as u8)
        };

        self.set_register_value("AL", quotient as u32)?;
        self.set_register_value("AH", remainder as u32)?;
        Ok(())
    }

    // DX:AX / src_value, DX = DX:AX % src_value
    fn div_16bit(&mut self, src_value: u16, signed: bool) -> Result<(), ErrorCode> {
        let ax_value = self.get_register_value("AX")?;
        let dx_value = self.get_register_value("DX")?;
        let dividend = ((dx_value as u32) << 16) | (ax_value as u32);
        if src_value == 0 {
            return Err(ErrorCode::DivisionByZero);
        }

        let (quotient, remainder) = if signed {
            let dividend_signed = dividend as i32;
            let src_signed = src_value as i16;

            let (q, r) = (dividend_signed / src_signed as i32, dividend_signed % src_signed as i32);
            (q as u16, r as u16)

        } else {
            ((dividend / src_value as u32) as u16, (dividend % src_value as u32) as u16)
        };

        self.set_register_value("AX", quotient as u32)?;
        self.set_register_value("DX", remainder as u32)?;
        Ok(())
    }

    fn div_32bit(&mut self, src_value: u32, signed: bool) -> Result<(), ErrorCode> {
        // Fetch EAX and EDX register values
        let eax_value = self.get_register_value("EAX")?;  // Assuming this retrieves u32
        let edx_value = self.get_register_value("EDX")?;  // Assuming this retrieves u32

        // Create the 64-bit dividend from EDX:EAX
        let dividend = ((edx_value as u64) << 32) | (eax_value as u64);

        // Handle division by zero
        if src_value == 0 {
            return Err(ErrorCode::DivisionByZero);
        }

        // Perform division and modulus based on signed or unsigned mode
        let ((quotient, remainder), overflowed) = if signed {
            // Signed division and modulus
            let dividend_signed = dividend as i64;
            let src_signed = src_value as i32;
            
            // Check for overflow in division (when dividing i64::MIN by -1)
            if src_signed == -1 && dividend_signed == i64::MIN {
                return Err(ErrorCode::Overflow);
            }

            let (q, overflow1) = dividend_signed.overflowing_div(src_signed as i64);
            let (r, overflow2) = dividend_signed.overflowing_rem(src_signed as i64);
            ((q as u32, r as u32), overflow1 || overflow2)
        } else {
            // Unsigned division and modulus
            let (q, overflow1) = dividend.overflowing_div(src_value as u64);
            let (r, overflow2) = dividend.overflowing_rem(src_value as u64);
            ((q as u32, r as u32), overflow1 || overflow2)
        };

        // Check for overflow and return error if necessary
        if overflowed {
            return Err(ErrorCode::Overflow);
        }

        // Update EAX and EDX registers with quotient and remainder
        self.set_register_value("EAX", quotient)?;
        self.set_register_value("EDX", remainder)?;
        Ok(())
    }
}

