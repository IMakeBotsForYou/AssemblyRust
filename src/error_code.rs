use std::fmt;

#[derive(Debug)]
pub enum ErrorCode {
    DivisionByZero,
    StackOverflow,
    StackUnderflow,
    InvalidOpcode,
    UnknownVariable,
    Overflow,
    InvalidRegister,
    VariableAlreadyExists,
    InvalidPointer(String),
    NotEnoughSpace(String),
    InvalidValue(String),
}


impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::DivisionByZero => write!(f, "Division By Zero"),
            ErrorCode::StackOverflow => write!(f, "Stack Overflow"),
            ErrorCode::StackUnderflow => write!(f, "Stack Underflow"),
            ErrorCode::InvalidOpcode => write!(f, "Invalid Opcode"),
            ErrorCode::UnknownVariable => write!(f, "Unknown Variable"),
            ErrorCode::Overflow => write!(f, "Overflow"),
            ErrorCode::InvalidRegister => write!(f, "Invalid Register"),
            ErrorCode::VariableAlreadyExists => write!(f, "Attempted to save variable/label that already exists"),
            ErrorCode::InvalidPointer(msg) => write!(f, "Invalid Pointer: {}", msg),
            ErrorCode::NotEnoughSpace(msg) => write!(f, "Not Enough Space: {}", msg),
            ErrorCode::InvalidValue(msg) => write!(f, "Invalid Value: {}", msg),
        }
    }
}