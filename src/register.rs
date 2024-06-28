
use crate::variable_metadata::VariableSize;
use crate::ErrorCode;
#[derive(Clone)]
pub struct Register {
    value: u32,
    pub name: RegisterName,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterName {
    EAX,
    AX,
    AL,
    AH,
    EBX,
    BX,
    BL,
    BH,
    ECX,
    CX,
    CL,
    CH,
    EDX,
    DX,
    DL,
    DH,
    ESI,
    SI,
    EDI,
    DI,
    BP,
    SP,
    IP,
    FLAG,
}
impl RegisterName {
    pub fn from_str(input: &str) -> Result<Self, ErrorCode> {
        match input {
            "EAX" => Ok(RegisterName::EAX),
            "AX" => Ok(RegisterName::AX),
            "AL" => Ok(RegisterName::AL),
            "AH" => Ok(RegisterName::AH),
            "EBX" => Ok(RegisterName::EBX),
            "BX" => Ok(RegisterName::BX),
            "BL" => Ok(RegisterName::BL),
            "BH" => Ok(RegisterName::BH),
            "ECX" => Ok(RegisterName::ECX),
            "CX" => Ok(RegisterName::CX),
            "CL" => Ok(RegisterName::CL),
            "CH" => Ok(RegisterName::CH),
            "EDX" => Ok(RegisterName::EDX),
            "DX" => Ok(RegisterName::DX),
            "DL" => Ok(RegisterName::DL),
            "DH" => Ok(RegisterName::DH),
            "ESI" => Ok(RegisterName::ESI),
            "SI" => Ok(RegisterName::SI),
            "EDI" => Ok(RegisterName::EDI),
            "DI" => Ok(RegisterName::DI),
            "BP" => Ok(RegisterName::BP),
            "SP" => Ok(RegisterName::SP),
            "IP" => Ok(RegisterName::IP),
            "FLAG" => Ok(RegisterName::FLAG),
            _ => Err(ErrorCode::InvalidRegister(
                format!("{} is an invalid register.", input)
            )),
        }
    }

    pub fn is_valid_name(name: &str) -> bool {
        match RegisterName::from_str(name) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn to_index(&self) -> usize {
        match self {
            RegisterName::EAX | RegisterName::AX | RegisterName::AL | RegisterName::AH => 0,
            RegisterName::EBX | RegisterName::BX | RegisterName::BL | RegisterName::BH => 1,
            RegisterName::ECX | RegisterName::CX | RegisterName::CL | RegisterName::CH => 2,
            RegisterName::EDX | RegisterName::DX | RegisterName::DL | RegisterName::DH => 3,
            RegisterName::ESI | RegisterName::SI => 4,
            RegisterName::EDI | RegisterName::DI => 5,
            RegisterName::BP => 6,
            RegisterName::SP => 7,
            RegisterName::IP => 8,
            RegisterName::FLAG => 9,
        }
    }
    pub fn is_top(&self) -> Result<bool, ErrorCode> {
        match self {
            RegisterName::AL | RegisterName::BL | RegisterName::CL | RegisterName::DL => Ok(false),
            RegisterName::AH | RegisterName::BH | RegisterName::CH | RegisterName::DH => Ok(true),
            _ => Err(ErrorCode::InvalidRegister(
                        format!(
                            "Register {:?} is not a single-byte register, thus it can't have a top-bottom",
                            self)
                        )
                    )
        }
    }

}

impl Register {
    pub fn new(name: RegisterName) -> Self {
        let index = &name.to_index();
        Self { value: 0, name: name, index: *index }
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

pub fn get_register_size(reg_name: &RegisterName) -> VariableSize {
    match reg_name {
        RegisterName::AL | RegisterName::BL | RegisterName::CL | RegisterName::DL |
        RegisterName::AH | RegisterName::BH | RegisterName::CH | RegisterName::DH => VariableSize::Byte,
        
        RegisterName::AX | RegisterName::BX | RegisterName::CX | RegisterName::DX |
        RegisterName::SI | RegisterName::DI | RegisterName::IP | RegisterName::FLAG |
        RegisterName::BP | RegisterName::SP => VariableSize::Word,

        RegisterName::EAX | RegisterName::EBX | RegisterName::ECX | RegisterName::EDX |
        RegisterName::ESI | RegisterName::EDI => VariableSize::DoubleWord
    }
}
