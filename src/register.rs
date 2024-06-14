
use crate::variable_metadata::VariableSize;

pub struct Register {
    value: u32,
    pub name: String,
}



impl Register {
    pub fn new(name: &str) -> Self {
        Self { value: 0, name: name.to_string() }
    }

    pub fn get_byte(&self, top: bool) -> u8 {
        if top {
            // return H
            ((self.value & 0x0000FF00) >> 8) as u8
        } else {
            // return L
            (self.value & 0x0000000FF) as u8
        }
    }

    pub fn get_word(&self) -> u16 {
        self.value as u16 // X
    }

    pub fn get_dword(&self) -> u32  {
        self.value
    }

    pub fn load_byte(&mut self, value: u8, top: bool) {
        // Clear correct byte
        let mask: u32 = 0xFF << if top {8} else {0};
        self.value &= !mask;

        // Set new value at byte
        let new_value = (value as u32) << if top {8} else {0};  
        self.value |= new_value;
    }

    pub fn load_word(&mut self, value: u16) {
        // Set -X to 0 and renew it with new value
        self.value = self.value & 0xFFFF0000 | value as u32;
    }

    pub fn load_dword(&mut self, value: u32) {
        self.value = value;
    }
}

pub fn get_register_size(reg_name: &str) -> Option<VariableSize> {
    match reg_name {
        "AL" | "BL" | "CL" | "DL" | "AH" | "BH" | "CH" | "DH" => Some(VariableSize::Byte),
        "AX" | "BX" | "CX" | "DX" | "SI" | "DI" | "IP" | "FLAG" => Some(VariableSize::Word),
        "EAX" | "EBX" | "ECX" | "EDX" | "ESI" | "EDI"  => Some(VariableSize::DoubleWord),
        _ => None
    }
}

pub fn get_register(name: &str) -> usize{
    match name {
        "EAX"|"AX"|"AL"|"AH" => 0,
        "EBX"|"BX"|"BL"|"BH" => 1,
        "ECX"|"CX"|"CL"|"CH" => 2,
        "EDX"|"DX"|"DL"|"DH" => 3,
        "SI" => 4,
        "DI" => 5,
        "IP" => 6,   
        "FLAG" => 7,   
        _ => panic!("Invalid register name: {}", name),
    }
}

