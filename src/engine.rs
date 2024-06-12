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
    memory_manager::MemoryManager,
    register::{
        get_register, 
        Register,
        get_register_size
    },
    status::Status,
    utils::{
        read_lines_from_file, 
        parse_string_to_usize
    },
    command::Command
};


pub struct Engine {
    lines: LineProcessor, // lines of source code (.txt)
    pub registers: [Register; 8], // A-D, ESI, EDI, P
    memory_manager: MemoryManager, // 16 KB bytes of memory
    // mode: bool, // false = reading data, true = reading code
    stack_pointer: usize, // pointer to the top of the stack within memory,
    labels: HashMap<String, usize>, // labels to jump to
    status: Status, // status: ok, error, halted,
    valid_registers: HashSet<String>,
    // interrupts: Vec<Interrupt>
}


impl Engine {
    pub fn new(file_name: &str) -> io::Result<Self> {
        let file_lines = read_lines_from_file(file_name)?;
        let my_registers: [Register; 8] = [
            Register::new("AX"),
            Register::new("BX"),
            Register::new("CX"),
            Register::new("DX"),
            Register::new("ESI"),
            Register::new("EDI"),
            Register::new("IP"),
            Register::new("FLAG"),
        ];
        const MEMORY_SIZE: usize = 1024 * 16; // 16 KB

        let valid_registers: HashSet<String> = [
            "AX", "BX", "CX", "DX", "ESI", "EDI", "IP", "FLAG",
            "AL", "AH", "BL", "BH", "CL", "CH", "DL", "DH",
        ].iter().cloned().map(String::from).collect();
            Ok(Self {
                lines: LineProcessor::new(file_lines),
                registers: my_registers,
                memory_manager: MemoryManager::new(MEMORY_SIZE),
                stack_pointer: MEMORY_SIZE-1, // Initialize stack pointer to the end of memory
                labels: HashMap::new(),
                status: Status::Ok,
                valid_registers,
                // Interrupt.new(),
            })
    }

    fn is_valid_register(&self, reg_to_check: &str) -> bool {
        self.valid_registers.contains(reg_to_check)
    }

    fn both_valid_reg(&self, reg_1: &str, reg_2: &str) -> bool {
        self.is_valid_register(reg_1) && self.is_valid_register(reg_2)
    }


    fn addition_mem(&mut self, dest: usize, value: u16, word: bool) -> Result<(), ErrorCode>{
        self.memory_manager.check_memory_address(dest)?; 

        if word {

            self.memory_manager.check_memory_address(dest+1)?;   

            let mem_top_byte = (self.memory_manager.get_byte(dest + 1)? as u16) << 8;
            let mem_bottom_byte = self.memory_manager.get_byte(dest)? as u16;

            let word_value = mem_top_byte | mem_bottom_byte;

            let (sum, overflowed) = word_value.overflowing_add(value);

            self.memory_manager.set_memory(dest, (sum >> 8) as u8);
            self.memory_manager.set_memory(dest+1, (sum & 0x0FF) as u8);

            self.set_flags(sum, overflowed);

        } else {
            
            if value > 127 {
                return Err(ErrorCode::InvalidValue(
                        format!("Single byte value can't be over 127. Attempted value is {}", value)
                        .to_string()
                    ));
            }

            let byte_value = self.memory_manager.get_byte(dest)?;

            let (sum, overflowed) = byte_value.overflowing_add(value as u8);

            self.memory_manager.set_memory(dest, sum);

            self.set_flags(sum as u16, overflowed);
        }

        return Ok(());
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
                let current_value = self.registers[dest_register].get_word();
                let (result, overflowed) = current_value.overflowing_add(value_to_add);
                self.registers[dest_register].load_word(result); // Load low byte into AH
                self.set_flags(result, overflowed);
            },
            _ => panic!("Invalid destination"),
        };
    }

    pub fn get_register_value(&self, name: &str) -> u16 {
        let value = self.registers[get_register(name)].get_word();

        match name {
            "AL" | "BL" | "CL" | "DL" => value & 0x00FF,
            "AH" | "BH" | "CH" | "DH" => value >> 8,
            "FLAG" | "ESI" | "EDI" | "IP" | "AX" | "BX" | "CX" | "DX"  => value, 
            _ => panic!("Invalid Register")
        }
    }

    fn set_flags(&mut self, result: u16, overflowed: bool) {

        self.set_register_value("FLAG", 0);

        if result.count_ones() % 2 == 0 {
            self.set_flag(Flag::Parity, true);

        }

        // Set Zero Flag
        if result == 0 {
            self.set_flag(Flag::Zero, true);    
        }

        // Set Sign Flag
        if result >> 15 == 1 {
            self.set_flag(Flag::Sign, true);    
        }

        // Set Overflow Flag
        if overflowed {
            self.set_flag(Flag::Overflow, true);    
        }
        
    }

    fn save_label(&mut self, name: String) -> Result<(), ErrorCode> {

        if self.labels.insert(name, self.get_register_value("IP") as usize).is_some() {
            return Err(ErrorCode::VariableAlreadyExists);
        } else {
            Ok(())
        }
    }

    pub fn execute(&mut self) -> Result<(), ErrorCode>{
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
            let maybe_arguments = self.lines.next(&mut self.registers[get_register("IP")], true);

            if maybe_arguments.is_none() {
                break;
            }
            let str_maybe_arguments = maybe_arguments.unwrap();
            let arguments: Vec<&str> = convert_vec_string_to_vec_str(&str_maybe_arguments);

            let ip = self.registers[get_register("IP")].get_word();

            // Process the instruction based on the arguments
            match arguments.as_slice() {
                ["mov", reg_dst, reg_src] if self.both_valid_reg(reg_dst, reg_src) => {
                    self.mov_reg_reg(reg_dst, reg_src)?;
                },
                
                ["lea", reg, mem_address] if self.is_valid_register(*reg) && self.memory_manager.is_memory_operand(mem_address) => {
                    match self.memory_manager.calculate_effective_address(mem_address, &self.registers, true){
                        Ok(parsed_address) => {
                            if get_register_size(*reg) == Some(8) {
                                return Err(ErrorCode::NotEnoughSpace(
                                    format!("Cannot store pointer in 1 byte register: {reg}."))
                                );
                            } else {
                                self.mov_reg_const(reg, parsed_address as u16);
                            }
                        },
                        Err(error) =>  return Err(error),
                    };

                },

                ["lea", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Lea));
                    self.halt();
                    return Err(ErrorCode::InvalidOpcode);
                }

                ["mov", reg, constant] if self.is_valid_register(*reg) && parse_string_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_string_to_usize(*constant) {
                        self.mov_reg_const(reg, value as u16);
                    } else {
                        println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },

                ["mov", reg, mem_address] if self.is_valid_register(*reg) && self.memory_manager.is_memory_operand(mem_address) => {
                    match self.memory_manager.calculate_effective_address(mem_address, &self.registers, true){
                        Ok(parsed_address) => {
                            if let Err(error) = self.mov_reg_mem(reg, parsed_address) {
                                self.halt();
                                return Err(error);
                            }
                        },
                        Err(error) =>  {
                            println!("[WARNING] [{ip}] Something went wrong. {:?}", error);
                        },
                    };

                },

                ["mov", mem_address, reg] if self.is_valid_register(*reg) && self.memory_manager.is_memory_operand(mem_address) => {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers, true) {
                        if let Err(error) = self.mov_mem_reg(parsed_address, reg) {
                            self.halt();
                            return Err(error);
                        }
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },
                
                ["mov", mem_address, constant] if parse_string_to_usize(*constant).is_some() && self.memory_manager.is_memory_operand(*mem_address) => {
                    if let Some(value) = parse_string_to_usize(*constant) {
                        if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(*mem_address, &self.registers, true) {
                            self.memory_manager.set_memory(parsed_address, value as u8);
                            self.set_flags(value as u16, false);
                        } else {
                             println!("[WARNING] [{ip}] Something went wrong.");
                        }
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },

                ["mov", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Mov));
                    self.halt();
                    return Err(ErrorCode::InvalidOpcode);
                },

                // ADD Instructions
                ["add", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                    self.add_reg_reg(reg_dst, reg_src)?;
                },
                ["add", reg_dst, mem_address] if self.memory_manager.is_memory_operand(mem_address) => {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers, true) {
                        if let Err(error) = self.add_reg_mem(reg_dst, parsed_address) {
                            self.halt();
                            return Err(error);
                        }
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },
                
                ["add", mem_address, reg_src] if self.memory_manager.is_memory_operand(mem_address) && self.is_valid_register(reg_src)=> {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers, true) {
                        if let Err(error) = self.add_mem_reg(parsed_address, reg_src) {
                            self.halt();
                            return Err(error);
                        }
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },
                ["add", reg, constant] if self.is_valid_register(*reg) && parse_string_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_string_to_usize(*constant) {
                        self.add_reg_const(reg, value as u16);
                    } else {
                        println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },

                ["add", mem_address, constant] if parse_string_to_usize(*constant).is_some() && self.memory_manager.is_memory_operand(*mem_address) => {
                    if let Some(value) = parse_string_to_usize(*constant) {
                        if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(*mem_address, &self.registers, true) {
                            let (result, overflowed) = self.memory_manager.get_byte(parsed_address)?.overflowing_add(value as u8);
                            self.set_flags(value as u16, overflowed);
                            self.memory_manager.set_memory(parsed_address, result);
                        } else {
                             println!("[WARNING] [{ip}] Something went wrong.");
                        }
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },



                ["add", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Add));
                    self.halt();
                    return Err(ErrorCode::InvalidOpcode);
                },

                // SUB Instructions
                ["sub", reg_dst, reg_src] if self.both_valid_reg(*reg_dst, *reg_src) => {
                    self.sub_reg_reg(reg_dst, reg_src)?;
                },

                ["sub", reg_dst, mem_address] if self.memory_manager.is_memory_operand(mem_address) => {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers, true) {
                        if let Err(error) = self.sub_reg_mem(reg_dst, parsed_address) {
                            self.halt();
                            return Err(error);
                            
                        }
                        
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },
                ["sub", mem_address, reg_src] if self.memory_manager.is_memory_operand(mem_address) && self.is_valid_register(reg_src)=> {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers, true) {
                        if let Err(error) = self.sub_mem_reg(parsed_address, reg_src) {
                            self.halt();
                            return Err(error);
                        }
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },
                ["sub", reg, constant] if self.is_valid_register(*reg) && parse_string_to_usize(*constant).is_some() => {
                    if let Some(value) = parse_string_to_usize(*constant) {
                        self.sub_reg_const(reg, value as u16);
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },


                ["sub", mem_address, constant] if parse_string_to_usize(*constant).is_some() && self.memory_manager.is_memory_operand(*mem_address) => {
                    if let Some(value) = parse_string_to_usize(*constant) {
                        if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(*mem_address, &self.registers, true) {
                            let (result, overflowed) = self.memory_manager.get_byte(parsed_address)?.overflowing_sub(value as u8);
                            self.set_flags(value as u16, overflowed);
                            self.memory_manager.set_memory(parsed_address, result);
                        } else {
                             println!("[WARNING] [{ip}] Something went wrong.");
                        }
                    } else {
                         println!("[WARNING] [{ip}] Something went wrong.");
                    }
                },

                ["sub", _rest @ ..] => {
                    println!("{}", Command::get_help_string(Command::Sub));
                    self.halt();
                    return Err(ErrorCode::InvalidOpcode);
                },


                [op @ ("mul" | "imul"), reg_src] if self.is_valid_register(*reg_src) => {
                    // source register
                    self.mul_reg(*reg_src, *op == "imul");
                },

                [op @ ("mul" | "imul"), _rest @ ..] => {
                    let signed = *op == "imul";
                    if signed {
                        println!("{}", Command::get_help_string(Command::Imul));
                    } else {
                        println!("{}", Command::get_help_string(Command::Mul));
                    }
                    self.halt();
                    return Err(ErrorCode::InvalidOpcode);
                },


                // // DIV Instructions

                [op @ ("div" | "idiv"), reg_src] if self.is_valid_register(*reg_src) => {
                    // source register
                    if let Err(error) = self.div_reg(*reg_src, *op == "idiv") {
                        self.halt();
                        return Err(error);
                    }
                },
  
                [op @ ("div" | "idiv"), _rest @ ..] => {
                    let signed = *op == "idiv";
                    if signed {
                        println!("{}", Command::get_help_string(Command::Idiv));
                    } else {
                        println!("{}", Command::get_help_string(Command::Div));
                    }
                    self.halt();
                    return Err(ErrorCode::InvalidOpcode);
                },

                // // PRINT  Instructions

                ["print", reg] if self.is_valid_register(*reg) => {
                    println!("\n[PRINT] [{ip}]: {0}\n", self.get_register_value(*reg));
                },

                ["print", mem_address] => { // if self.memory_manager.is_memory_operand(mem_address) => {
                    
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers, true) {
                        if let Err(error) = self.memory_manager.check_memory_address(parsed_address) {
                            self.halt();
                            return Err(error);
                        }
                        let ip = self.registers[get_register("IP")].get_word();

                        println!("\n[PRINT] [{ip}]: {0}\n", self.memory_manager.get_byte(parsed_address)?);

                    }
                },
                ["print", number, mem_address]  => { //if self.memory_manager.is_memory_operand(mem_address) &&
                                                                                            //parse_string_to_usize(*number).is_some() => {
                    if let Ok(parsed_address) = self.memory_manager.calculate_effective_address(mem_address, &self.registers, true) {
                        if let Some(value) = parse_string_to_usize(*number) {
                    
                            if let Err(error) = self.memory_manager.check_memory_address(parsed_address+value) {
                                self.halt();
                                return Err(error);
                            }

                            let ip = self.registers[get_register("IP")].get_word();
                            print!("[PRINT] [{ip}]: [ ");
                            for i in 0..value {
                                print!("{0} ", self.memory_manager.get_byte(parsed_address+i)?);
                            }
                            println!("]");
                        }
                    }
                },
                ["int", interrupt] => if parse_string_to_usize(*interrupt) == Some(0){

                }
                ["NOP"] => {},
                [variable_name, define_as @ ("db"|"dw"), rest @ ..] if self.memory_manager.is_valid_variable_name(*variable_name) => {
                // (parse_string_to_usize(*data).is_some() || self.memory_manager.is_valid_array(*data).is_ok() )=> {
                    let size = if *define_as == "db" {1} else {2};

                    let mut bytes: Vec<u16> = Vec::new();

                    for (_, &arg) in rest.iter().enumerate() {
                        // Check if argument is a string literal and remove surrounding quotes
                        if let Some((start_char, end_char)) = arg.chars().next().zip(arg.chars().rev().next()) {
                            if (start_char == '"' && end_char == '"') || (start_char == '\'' && end_char == '\'') {
                                let inner = &arg[1..arg.len() - 1];
                                // bytes.extend_from_slice(inner.as_bytes());
                                for c in inner.chars() {
                                    bytes.push(c as u16);
                                }
                            } else {
                                // Handle other cases (numeric values, etc.)
                                if let Some(value) = parse_string_to_usize(arg) {
                                    if size == 1 {
                                        bytes.push(value as u8 as u16);
                                    } else if size == 2 {
                                        bytes.push(value as u16);
                                    }
                                }
                            }
                        } else {
                            // Handle other cases (numeric values, etc.)
                            if let Some(value) = parse_string_to_usize(arg) {
                                if size == 1 {
                                    bytes.push(value as u8 as u16);
                                } else if size == 2 {
                                    bytes.push(value as u16);
                                }                           
                             }
                        }
                    }
                    if let Err(error) = self.memory_manager.save_variable(variable_name.to_string(), &bytes, self.stack_pointer, size) {
                        self.halt();
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
                            match self.memory_manager.calculate_effective_address(&arg1, &self.registers, false) {
                                Ok(value) => first_value = value as isize,
                                Err(error) => return Err(error),
                            }
                        } else if let Some(v) = self.memory_manager.parse_value(&arg1, is_negative_first, &self.registers, false) {
                            first_value = v as isize;
                        } else {
                            return Err(ErrorCode::InvalidValue(format!("Could not parse {}", first_operand)));
                        }
                    
                        // Process second operand
                        if self.memory_manager.is_memory_operand(&arg2) {
                            match self.memory_manager.calculate_effective_address(&arg2, &self.registers, false) {
                                Ok(value) => second_value = value as isize,
                                Err(error) => return Err(error),
                            }
                        } else if let Some(v) = self.memory_manager.parse_value(&arg2, is_negative_second, &self.registers, false) {
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

                [label] => if self.labels.get(*label).is_some() {}
                _ => {
                    println!("Unknown instruction: {:?}", arguments);
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
            return Err(ErrorCode::InvalidPointer(
                format!("Label \"{}\" cannot be found.", label)
            ));
        }
        Ok(())
    }
    fn is_flag_on(&self, flag: Flag) -> bool {
        self.get_register_value("FLAG") & flag.value() != 0
    }

    fn check_register_sizes(&self, dest: &str, src: &str) -> Result <(), ErrorCode>{
        let size_dest = get_register_size(dest);
        let size_src = get_register_size(src);

        if size_src != size_dest {
            Err(ErrorCode::InvalidRegister)
        } else {
            Ok(())
        }
    }

    fn halt(&mut self){
        self.status = Status::Halted;
    }


    // MOV operations
    // REG <- Reg, Const, Var, Mem
    // Mem <- Reg, Const, Var, Mem
    // Var <- Reg, Const, Var, Mem
    // Const <- NOTHING, Const can't be moved into

    fn mov_reg_reg(&mut self, dest: &str, src: &str) -> Result<(), ErrorCode>{
        self.check_register_sizes(dest, src)?;

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
        Ok(())

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
    

    fn mov_reg_mem(&mut self, dest: &str, mem_address: usize) -> Result<(), ErrorCode> {
        let result: u16 = match dest.chars().last() {
            Some('L')|Some('H') => {
                if let Err(error) = self.memory_manager.check_memory_address(mem_address) {
                    return Err(error);
                }
                let is_top = dest.chars().last() == Some('H');
                let r = self.memory_manager.get_byte(mem_address)?;
                self.registers[get_register(dest)].load_byte(r, is_top);
                r as u16
            },
            Some('X') | Some('I') => {
                let result = self.memory_manager.get_word(mem_address)?;
                self.registers[get_register(dest)].load_word(result);
                result
            },
            _ => {0}
        };
        self.set_flags(result, false);
        Ok(())
    }

    fn mov_mem_reg(&mut self, mem_address: usize, src: &str) -> Result<(), ErrorCode>{
        match src.chars().last() {
            Some('L')|Some('H') => {
                if let Err(error) = self.memory_manager.check_memory_address(mem_address) {
                    return Err(error);
                }
                let value = self.get_register_value(src);
                self.memory_manager.set_memory(mem_address, value as u8);
                self.set_flags(value, false);                
            },
            Some('X') | Some('I') => {
                if let Err(error) = self.memory_manager.check_memory_address(mem_address+1) {
                    return Err(error);
                }
                self.memory_manager.set_memory(mem_address,      self.registers[get_register(src)].get_byte(true));
                self.memory_manager.set_memory(mem_address + 1, self.registers[get_register(src)].get_byte(false));
                self.set_flags(self.registers[get_register(src)].get_word(), false);
            },
            _ => println!("[ERROR] Invalid register")
        }
        Ok(())
    }

    fn add_reg_reg(&mut self, dest: &str, src: &str) -> Result<(), ErrorCode> {
        if let Err(error) = self.check_register_sizes(dest, src) {
            return Err(error);
        }
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
        Ok(())

    }

    fn add_mem_reg(&mut self, mem_address: usize, src: &str) -> Result<(), ErrorCode>{
        match src.chars().last() {
            Some('L') | Some('H') => {
                if let Err(error) = self.memory_manager.check_memory_address(mem_address) {
                    return Err(error);
                }
                let is_top = src.chars().last() == Some('H');
                let value = self.registers[get_register(src)].get_byte(is_top);
                if let Err(error) = self.addition_mem(mem_address, value as u16, false) {
                    return Err(error);
                }
            },
            Some('X') | Some('I') => {
                if let Err(error) = self.memory_manager.check_memory_address(mem_address+1) {
                    return Err(error);
                }
                let value = self.registers[get_register(src)].get_word();
                if let Err(error) = self.addition_mem(mem_address, value as u16, true) {
                    return Err(error);
                }            
            },
            _ => println!("[ERROR] Invalid register")
        }
        Ok(())
    }
    
    fn add_reg_mem(&mut self, dest: &str, mem_address: usize)  -> Result<(), ErrorCode>{
        self.memory_manager.check_memory_address(mem_address)?;
    
        let (result, overflowed) = {
            match dest.chars().last() {
                Some('X') | Some('I') => {
                    self.memory_manager.check_memory_address(mem_address+1)?;
                    let dest_value = self.registers[get_register(dest)].get_word();
                    let mem_value = self.memory_manager.get_word(mem_address)?;
                    dest_value.overflowing_add(mem_value)
                },
                Some('L') | Some('H') => {
                    let is_top = dest.chars().last() == Some('H');
                    let dest_value = self.registers[get_register(dest)].get_byte(is_top);
                    let mem_value = self.memory_manager.get_byte(mem_address)?;
                    let (sum, overflowed) = dest_value.overflowing_add(mem_value);
                    (sum as u16, overflowed)
                },
                _ => (0, false),
            }
        };
    
        self.set_flags(result, overflowed);
        self.mov_reg_const(dest, result);
        Ok(())
    }

    
    //////////// SUB /////////////
    fn sub_reg_reg(&mut self, dest: &str, src: &str) -> Result<(), ErrorCode> {
        if let Err(error) = self.check_register_sizes(dest, src) {
            return Err(error);
        }

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

        Ok(())

    }

    fn sub_mem_reg(&mut self, mem_address: usize, src: &str) -> Result<(), ErrorCode> {
        match src.chars().last() {
            Some('L') | Some('H') => {
                if let Err(error) = self.memory_manager.check_memory_address(mem_address) {
                    return Err(error);
                }
                let is_top = src.chars().last() == Some('H');
                let value = self.registers[get_register(src)].get_byte(is_top);
                if let Err(error) = self.addition_mem(mem_address, !value as u16, false) {
                    return Err(error);
                }
            },
            Some('X') | Some('I') => {
                if let Err(error) = self.memory_manager.check_memory_address(mem_address+1) {
                    return Err(error);
                }
                let value = self.registers[get_register(src)].get_word();
                if let Err(error) = self.addition_mem(mem_address, !value as u16, true) {
                    return Err(error);
                }            
            },
            _ => {println!("[ERROR] Invalid register");}
        }
        Ok(())
    }
    
    fn sub_reg_mem(&mut self, dest: &str, mem_address: usize) -> Result<(), ErrorCode> {
        self.memory_manager.check_memory_address(mem_address)?;
        self.memory_manager.set_memory(mem_address, !self.memory_manager.get_byte(mem_address)?+1);
        self.add_reg_mem(dest, mem_address)?;
        self.memory_manager.set_memory(mem_address, !self.memory_manager.get_byte(mem_address)?+1);
        Ok(())
    }
    
    fn sub_reg_const(&mut self, dest: &str, constant: u16) {
        let dest_value = self.registers[get_register(dest)].get_word();
        let (result, overflowed) = match dest.chars().last() {
            Some('X') | Some('I') => {
                let (value, overflowed) = dest_value.overflowing_sub(constant);
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

    fn set_register_value(&mut self, register: &str, value: u16) {
        match get_register_size(register) {
            Some(8) => self.registers[get_register(register)].load_byte(value.try_into().unwrap(), register.ends_with('H')),
            Some(16) => self.registers[get_register(register)].load_word(value),
            None => panic!("Invalid register."),
            _ => panic!("Invalid size."),
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
    fn mul_reg(&mut self, reg_src: &str, signed: bool) {
        let src_value = self.get_register_value(reg_src);
        // Check the size of the source register.
        if get_register_size(reg_src) == Some(8) {
            self.mul_8bit(src_value, signed);
        } else {
            self.mul_16bit(src_value, signed);
        }
    }

    // Helper function to multiply 8-bit values and store the result in AX register.
    fn mul_8bit(&mut self, src_value: u16, signed: bool) {
        // Get the current value of the AX register
        let ax_value = self.get_register_value("AX");

        // Extract the lower 8 bits of AX (AL) and convert to signed 8-bit value
        let al_value = ax_value as i8;
        let src_value = src_value as i8;

        // Perform multiplication based on signed or unsigned interpretation
        let result = if signed {
            (al_value as i16 * src_value as i16) as u16
        } else {
            (ax_value & 0x00FF) as u16 * (src_value as u16 & 0x00FF) as u16
        };

        // Determine overflow condition based on the upper nibble of the result
        let overflow_condition = result >> 4 != 0;

        self.set_flag(Flag::Carry, overflow_condition);
        self.set_flag(Flag::Overflow, overflow_condition);
        self.set_register_value("AX", result as u16);
    }


    fn mul_16bit(&mut self, src_value: u16, signed: bool) {
        // Get the current value of the AX register
        let ax_value = self.get_register_value("AX");
    
        // Perform multiplication based on signed or unsigned interpretation
        let result = if signed {
            // Signed multiplication
            let result = (ax_value as i32).checked_mul(src_value as i32).expect("Overflow error");
            result as u32
        } else {
            // Unsigned multiplication
            (ax_value as u32).checked_mul(src_value as u32).expect("Overflow error")
        };
        
        // Store the lower 16 bits of the result in AX
        self.set_register_value("AX", (result & 0xFFFF) as u16);
        // Store the upper 16 bits of the result in DX
        self.set_register_value("DX", ((result >> 16) & 0xFFFF) as u16);
    
        // Determine overflow condition
        let dx_value = (result >> 16) & 0xFFFF;
        let overflow_condition = dx_value != 0;
        self.set_flag(Flag::Overflow, overflow_condition);
        self.set_flag(Flag::Carry, overflow_condition);
    }



    //////////// DIV ////////////
    // Divide the value in the source register by the value in AX register.
    fn div_reg(&mut self, reg_src: &str, signed: bool) -> Result<(), ErrorCode>{
        let src_value = self.get_register_value(reg_src);
        // Check the size of the source register.
        if get_register_size(reg_src) == Some(8) {
            self.div_8bit(src_value as u8, signed)
        } else {
            self.div_16bit(src_value, signed)
        }
    }
    // ax / bl, ah = ax % bl
    fn div_8bit(&mut self, src_value: u8, signed: bool) -> Result<(), ErrorCode>{
        let ax_value = self.get_register_value("AX");
        if src_value == 0 {
            return Err(ErrorCode::DivisionByZero);
        }
        let (quotient, remainder) = if signed {
            // Perform signed division
            let al_signed = ax_value as i16;
            let src_signed = src_value as i16;
            let quotient = (al_signed / src_signed) as u16;
            let remainder = (al_signed % src_signed) as u16;
            (quotient, remainder)
        } else {
            // Perform unsigned division
            let quotient = ax_value / src_value as u16;
            let remainder = ax_value % src_value as u16;
            (quotient, remainder)
        };
        self.set_register_value("AL", quotient as u16);
        self.set_register_value("AH", remainder as u16);
        Ok(())
    }


    // Helper function to divide 16-bit values and store the result in AX and DX registers.
    fn div_16bit(&mut self, src_value: u16, signed: bool) -> Result<(), ErrorCode>{
        let ax_value = self.get_register_value("AX");
        if src_value == 0 {
            return Err(ErrorCode::DivisionByZero);
        }
        let (quotient_maybe, remainder) = if signed {
            // Perform signed division
            let al_signed = ax_value as i16;
            let src_signed = src_value as i16;
            let quotient = (al_signed as u16).checked_div(src_signed as u16);
            let remainder = (al_signed % src_signed) as u16;
            (quotient, remainder)
        } else {
            // Perform unsigned division
            let quotient = ax_value.checked_div(src_value);
            let remainder = ax_value % src_value;
            (quotient, remainder)
        };
        if quotient_maybe.is_none() {
            return Err(ErrorCode::Overflow);
        }

        let quotient = quotient_maybe.expect("Some error occurred during unwrapping of quotient");

        // Check for quotient overflow
        if quotient > u16::MAX {
            return Err(ErrorCode::Overflow);
        }


        self.set_register_value("AX", quotient);
        self.set_register_value("DX", remainder);

        if signed {
            // Check if the result fits in the lower half (AX) without overflow
            let result_sign_extend = self.get_register_value("AH") == 0;
            let overflow_condition = !result_sign_extend;
            self.set_flag(Flag::Overflow, overflow_condition);
            self.set_flag(Flag::Carry, overflow_condition);
        } else {
            // Check if the upper half (DX) is zero
            let dx_value = self.get_register_value("DX");
            self.set_flag(Flag::Overflow, dx_value != 0);
            self.set_flag(Flag::Carry, dx_value != 0);
        }

        Ok(())
    }
}

