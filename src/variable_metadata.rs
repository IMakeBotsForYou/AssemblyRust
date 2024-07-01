use crate::ErrorCode;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum VariableSize {
    Byte = 1,
    Word = 2,
    DoubleWord = 4,
}
#[derive(Copy, Clone, Debug)]
pub struct VariableMetadata {
    pub start_index: usize,
    pub length: usize,
    pub size: VariableSize,
}

impl VariableMetadata {
    pub fn new(start_index: usize, length: usize, size: VariableSize) -> Self {
        return Self {
            start_index,
            length,
            size,
        };
    }
}

impl VariableSize {
    pub fn from_usize(value: usize) -> Result<Self, ErrorCode> {
        match value {
            1 => Ok(VariableSize::Byte),
            2 => Ok(VariableSize::Word),
            4 => Ok(VariableSize::DoubleWord),
            _ => Err(ErrorCode::InvalidValue(format!("Value {value} doesn't correspond to a variable size\n1: Byte\n2: Word\n4: DoubleWord")))
        }
    }
    pub fn value(&self) -> usize {
        match self {
            VariableSize::Byte => 1,
            VariableSize::Word => 2,
            VariableSize::DoubleWord => 4,
        }
    }
    pub fn as_string(&self) -> String {
        match self {
            VariableSize::Byte => "Byte".to_string(),
            VariableSize::Word => "Word".to_string(),
            VariableSize::DoubleWord => "Double Word".to_string(),
        }
    }
}
