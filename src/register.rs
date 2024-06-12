
pub struct Register {
    value: u16,
    pub name: String,
}



impl Register {
    pub fn new(name: &str) -> Self {
        Self { value: 0, name: name.to_string() }
    }

    pub fn get_byte(&self, top: bool) -> u8 {
        if top {
            // return H
            (self.value >> 8) as u8
        } else {
            // return L
            (self.value & 0x00FF) as u8   
        }
    }

    pub fn get_word(&self) -> u16 {
        self.value
    }

    pub fn load_byte(&mut self, value: u8, top: bool) {
        if top {
            self.value = (self.value & 0x00FF) | ((value as u16) << 8);
        } else {
            self.value = (self.value & 0xFF00u16 as u16) | (value as u16 & 0x00FF);
        }
    }

    pub fn load_word(&mut self, value: u16) {
        self.value = value;
    }
}

pub fn get_register_size(reg_name: &str) -> Option<usize> {
    match reg_name {
        "AL" | "BL" | "CL" | "DL" | "AH"  | "BH"  | "CH" | "DH" => Some(8),
        "AX" | "BX" | "CX" | "DX" | "ESI" | "EDI" | "IP" | "FLAG" => Some(16),
        _ => None
    }
}

pub fn get_register(name: &str) -> usize{
    match name {
        "AX"|"AL"|"AH" => 0,
        "BX"|"BL"|"BH" => 1,
        "CX"|"CL"|"CH" => 2,
        "DX"|"DL"|"DH" => 3,
        "ESI" => 4,
        "EDI" => 5,
        "IP" => 6,   
        "FLAG" => 7,   
        _ => panic!("Invalid register name: {}", name),
    }
}

