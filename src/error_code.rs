use std::fmt;

#[derive(Debug)]
pub enum ErrorCode {
    DivisionByZero,
    StackOverflow,
    StackUnderflow,
    InvalidOpcode(String),
    UnknownVariable,
    Overflow,
    InvalidRegister(String),
    LabelAlreadyExists(String),
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
            ErrorCode::InvalidOpcode(msg) => write!(f, "Invalid Opcode: {msg}"),
            ErrorCode::UnknownVariable => write!(f, "Unknown Variable"),
            ErrorCode::Overflow => write!(f, "Overflow"),
            ErrorCode::InvalidRegister(msg) => write!(f, "Invalid Register: {msg}"),
            ErrorCode::LabelAlreadyExists(msg) => write!(f, "Attempted to save variable/label that already exists: {msg}"),
            ErrorCode::InvalidPointer(msg) => write!(f, "Invalid Pointer: {}", msg),
            ErrorCode::NotEnoughSpace(msg) => write!(f, "Not Enough Space: {}", msg),
            ErrorCode::InvalidValue(msg) => write!(f, "Invalid Value: {}", msg),
        }
    }
}