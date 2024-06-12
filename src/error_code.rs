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
