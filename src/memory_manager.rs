use crate::{
    error_code::ErrorCode, 
    variable_metadata::{
        VariableMetadata,
        VariableSize,
    }, 
    register::{
        get_register, 
        Register,
        get_register_size
    }, 
    utils::parse_string_to_usize};

use std::collections::HashMap;
use regex::Regex;


pub struct MemoryManager {
    memory: Vec<u8>,
    pub variable_pointers: HashMap<String, VariableMetadata>,
    pub labels: HashMap<String, usize>,
    pub procs: HashMap<String, (usize, usize)>,
    segments: [usize; 3],
}

impl MemoryManager {
    pub fn new(size: usize, seg: [usize; 3]) -> Self {
        MemoryManager {
            memory: vec![0; size],
            variable_pointers: HashMap::new(),
            labels: HashMap::new(),
            procs: HashMap::new(),
            segments: seg,
        }
    }
    
    pub fn get_variable(&self, variable_name: &str) -> Option<&VariableMetadata> {
        self.variable_pointers.get(variable_name)
    }

    pub fn check_memory_address(&self, mem_address: usize) -> Result<(), ErrorCode> {
        if mem_address >= self.memory.len() {
            Err(ErrorCode::InvalidPointer(format!(
                "{} is not a valid memory address", mem_address
            )))
            } else {
            Ok(())
        }
    }
    
    pub fn push_to_stack(&mut self, value: u32, size: VariableSize, si_register: &mut Register) -> Result<(), ErrorCode> {
        match size {
            VariableSize::Byte => {
                let si = si_register.get_word() as usize;
                self.set_byte(self.memory.len()-si-1, value as u8)?;
                si_register.load_word(si as u16 + 1);
            }
            VariableSize::Word => {
                let si = si_register.get_word() as usize;
                self.set_word(self.memory.len()-si-2, value as u16)?;
                si_register.load_word(si as u16 + 2);
            },
            VariableSize::DoubleWord => {
                let si: usize = si_register.get_word() as usize;
                self.set_dword(self.memory.len()-si-4, value)?;
                si_register.load_word(si as u16 + 4);
            }
        }
        Ok(())
    }
    pub fn pop_from_stack(&mut self, size: VariableSize, si_register: &mut Register) -> Result<u32, ErrorCode> {
        let result = match size {
            VariableSize::Byte => {
                let si = si_register.get_word() as usize;
                let value = self.get_byte(self.memory.len()-si)? as u32;
                si_register.load_word(si as u16 - 1);
                value
            },
            VariableSize::Word => {
                let si = si_register.get_word() as usize;
                let value = self.get_word(self.memory.len()-si)? as u32;
                si_register.load_word(si as u16 - 2);
                value
            },
            VariableSize::DoubleWord => {
                let si = si_register.get_word() as usize;
                let value = self.get_dword(self.memory.len()-si)?;
                si_register.load_word(si as u16 - 4);
                value
            }
        };
        Ok(result)
    }
    pub fn save_label(&mut self, name: String, ip: usize) -> Result<(), ErrorCode> {
        let name_copy = name.clone();
        if self.labels.insert(name, ip).is_some() {
            return Err(ErrorCode::LabelAlreadyExists(name_copy));
        } else {
            Ok(())
        }
    }
    pub fn save_proc(&mut self, name: String, start_ip: usize, end_ip: usize) -> Result<(), ErrorCode> {
        let name_copy = name.clone();
        if self.procs.insert(name, (start_ip, end_ip)).is_some() {
            return Err(ErrorCode::LabelAlreadyExists(name_copy));
        } else {
            Ok(())
        }
    }
    pub fn save_variable(&mut self, variable_name: String, data: &[u32], size: VariableSize) -> Result<(), ErrorCode> {
        let multiplier = size.value();

        let length = data.len() * multiplier;

        if self.variable_pointers.get(&variable_name).is_some() {
            return Err(ErrorCode::LabelAlreadyExists(variable_name));
        }
        if let Ok(location) = self.find_free_block(length) {
            // Save the metadata with the correct start_index
            println!("[SAVED] Saved variable {} @ {}\n", variable_name, location);
            self.variable_pointers.insert(variable_name, VariableMetadata::new(
                location,
                length,
                size,
            ));

            // Copy data to the found location
            for (i, &byte) in data.iter().enumerate() {
                match size {
                    VariableSize::Byte => self.memory[location + i] = byte as u8,
                    VariableSize::Word => {
                        self.memory[location+i*multiplier] = (byte >> 8) as u8;
                        self.memory[location+i*multiplier+1] = (byte & 0x00FF) as u8;
                    },
                    VariableSize::DoubleWord => {
                        self.memory[location+i*multiplier] =   (byte >> 24) as u8;
                        self.memory[location+i*multiplier+1] =   ((byte & 0x00FF0000) >> 16) as u8;
                        self.memory[location+i*multiplier+2] =   ((byte & 0x0000FF00) >> 8) as u8;
                        self.memory[location+i*multiplier+3] = byte as u8;
                    },
                }
            }


            
            Ok(())
        } else {
            Err(ErrorCode::NotEnoughSpace(
                format!("Not enough contiguous free memory to store variable of length {}", length)
            ))
        }
    }
    
    fn get_code_segment_displacement(&self) -> usize {
        self.segments[1]
    }
    
    pub fn find_free_block(&mut self, length: usize) -> Result<usize, ErrorCode> {
        let mut start_index = 0;

        // If there's no variables yet, return the first available space    
        if self.variable_pointers.len() == 0 {
            return Ok(0);
        }

        // Iterate over the variable pointers hashmap
        let mut entries: Vec<_> = self.variable_pointers.iter().collect();
        // Sort to guarantee consistent results
        entries.sort_by_key(|(_, metadata)| metadata.start_index);


        for (_, metadata) in entries {
            // Check if there's enough contiguous free memory between allocated blocks

            let end_index = metadata.start_index + metadata.length;

            // Check if there's enough free space between end of previous block and start of current block
            if start_index + length <= metadata.start_index {
                return Ok(start_index);
            }

            start_index = end_index;  // Move start_index to end of current block
            if let Err(error) = self.check_memory_address(start_index) {
                return Err(error);
            }

        }

        if start_index + length < self.get_code_segment_displacement()  {
            return Ok(start_index);
        }
        
        Err(ErrorCode::NotEnoughSpace(
            format!("Not enough contiguous free memory to store byte array of length {}", length)
            .to_string())
        )
    }

    pub fn is_valid_variable_name(&self, text: &str) -> bool {
        let variable_pattern = Regex::new(r"^([a-zA-Z_]+)$").unwrap();
        if let Some(captures) = variable_pattern.captures(text) {
            if let Some(_) = captures.get(1) {
                return true;
            }
        }
        return false;
    }

    pub fn is_memory_operand(&self, operand: &str) -> bool {
        operand.starts_with('[') && operand.ends_with(']')
    }

    pub fn get_register_value(&self, registers: &[Register; 10], name: &str) -> Option<u32> {

        if get_register_size(name).is_none() {
            return None;
        }

        let value = registers[get_register(name)].get_dword();

        match name {
            "AL"   | "BL"   | "CL"  | "DL" => Some(value & 0x000000FF),
            "AH"   | "BH"   | "CH"  | "DH" => Some(value & 0x0000FF00),
            "AX"   | "BX"   | "CX"  | "DX"  | "SI"  | "DI"  | "IP" | "FLAG" => Some(value & 0x0000FFFF),
            "EAX"  | "EBX"  | "ECX" | "EDX" | "ESI" | "EDI" => Some(value),
            _ => None,
        }
    }
    /* 
    Effective Address calculation follows this format:

    Effective Address=Base+(Index*Scale)+Displacement

     */
    pub fn calculate_effective_address(&self, mem_operand: &str, registers: &[Register; 10], label_vars: bool) -> Result<usize, ErrorCode> {
        // Ensure the memory operand is valid and remove the square brackets
        if !self.is_memory_operand(mem_operand) {
            return Err(ErrorCode::InvalidPointer("Memory Operand must be enveloped in []".to_string()));
        }
        let addr_expression = &mem_operand[1..mem_operand.len() - 1];
    
        let mut effective_address = 0;

        // Split the address expression into parts and process each part
        for part in addr_expression.split(|c| c == '+' || c == '-') {
            let part = part.trim();
            //allow spaces                                                                          don't underflow
            let is_negative = addr_expression.chars().nth(addr_expression.find(part).unwrap().saturating_sub(1)) == Some('-');
            
            // Process parts containing multiplication (index * scale)
            if part.contains('*') {

                let mut components = part.split('*').map(|s| s.trim());
                let index_part = components.next().ok_or(ErrorCode::InvalidPointer("Invalid Addressing Mode.".to_string()))?;
                let scale_part = components.next().ok_or(ErrorCode::InvalidPointer("Invalid Addressing Mode.".to_string()))?;
    
                // Get the index value from registers or as a direct value
                let index_value = if let Some(v) = self.get_register_value(registers, index_part) {
                    v as usize
                } else {
                    parse_string_to_usize(index_part).ok_or(ErrorCode::InvalidRegister(index_part.to_string()))? as usize
                };
    
                // Parse the scale value
                let scale_value = parse_string_to_usize(scale_part).ok_or(
                    ErrorCode::InvalidValue(
                        format!("Invalid scale factor: {scale_part}")
                    ))?;
    
                // Adjust the effective address based on the scale and sign
                effective_address += if is_negative {
                    - ((index_value * (scale_value as usize)) as isize)
                } else {
                    (index_value * (scale_value as usize)) as isize
                };
    
            // Handle displacement, hexadecimal values, or variable names
            } else if let Some(value) = self.parse_value(part, is_negative, registers, label_vars) {
                effective_address += value;
            } else {
                return Err(ErrorCode::InvalidRegister(part.to_string()));
            }
        }
    
        // // Ensure the effective address is positive and cast to usize
        // if effective_address < 0 {
        //     return Err(ErrorCode::InvalidPointer("Pointer address cannot be less than 0.".to_string()));
        // }
    
        Ok(effective_address as usize)
    }
    
    pub fn parse_value(&self, part: &str, is_negative: bool, registers: &[Register; 10], label_vars: bool) -> Option<isize> {
    
        if let Some(value) = parse_string_to_usize(part) {
            if is_negative {
                Some(-(value as isize))
            } else {
                Some(value as isize)
            }

        } else if let Some(var_metadata) = self.variable_pointers.get(part) {
            // Handle variable name as a pointer and adjust the effective address based on the sign
            if label_vars {
                let start_index = var_metadata.start_index as isize;
                if is_negative {
                    Some(-(start_index as isize))
                } else {
                    Some(start_index as isize)
                }
            } else {
                let value: u16;
                if var_metadata.length == 1 {
                    value = self.memory[var_metadata.start_index] as u16;
                } else {
                    value = (self.memory[var_metadata.start_index] as u16) << 8 | self.memory[var_metadata.start_index+1] as u16;
                }
                Some(value as isize)
            }

        // Handle base register and adjust the effective address based on the sign
        } else if let Some(v) = self.get_register_value(registers, part) {
            if is_negative {
                Some(-(v as isize))
            } else {
                Some(v as isize)
            }
        } else if let Some(v) = self.labels.get(part) {
                return Some(*v as isize);
        } else {
            None
        }
    }

    pub fn set_byte(&mut self, index: usize, value: u8) -> Result<(), ErrorCode> {
        self.check_memory_address(index)?;
        self.memory[index] = value;
        Ok(())
    }

    pub fn set_word(&mut self, index: usize, value: u16) -> Result<(), ErrorCode> {        
        self.check_memory_address(index)?; // Lower Bound
        self.check_memory_address(index+1)?; // Upper Bound
        self.memory[index] = (value >> 8) as u8;
        self.memory[index+1] = (value & 0x00FF) as u8;
        Ok(())
    }

    pub fn set_dword(&mut self, index: usize, value: u32)-> Result<(), ErrorCode>  {

        self.check_memory_address(index)?;   // Lower bound 
        self.check_memory_address(index+3)?; // Upper Bound
        // This is simpler.
        self.memory[index] =    ((value & 0xFF000000) >> 24) as u8;
        self.memory[index+1] = ((value & 0x00FF0000) >> 16) as u8;
        self.memory[index+2] = ((value & 0x0000FF00) >> 8) as u8;
        self.memory[index+3] =  (value & 0x000000FF) as u8;

        // for i in 0..4 {
        //     let mask = 0xFF000000 >> (i * 8); // Make mask for the byte we want
        //     let masked_value = value & mask;  // Mask everything else out
        //     let shifted_value = (masked_value >> ((3-i) * 8)) as u8; // Convert that byte to a u8
        //     self.memory[index+i] = shifted_value; // Set it
        // }

        Ok(())
    }
    pub fn get_byte(&self, index: usize) -> Result<u8, ErrorCode> {
        self.check_memory_address(index)?;
        Ok(self.memory[index])
    }

    pub fn get_word(&self, index: usize) -> Result<u16, ErrorCode>  {
        self.check_memory_address(index)?;
        self.check_memory_address(index+1)?; 
        Ok((self.memory[index] as u16) << 8 | self.memory[index+1] as u16)
    }

    pub fn get_dword(&self, index: usize) -> Result<u32, ErrorCode>  {
        for i in 0..4 {
            self.check_memory_address(index+i)?;
        }
        Ok(
            (self.memory[index  ] as u32) << 24 |
            (self.memory[index+1] as u32) << 16 |
            (self.memory[index+2] as u32) << 8 |
            (self.memory[index+3] as u32)
        )
    }
    pub fn _get_memory(&self, start_index: usize, amount: usize) -> Vec<u8> {
        self.memory[start_index..amount+start_index].to_vec()
    }
}
