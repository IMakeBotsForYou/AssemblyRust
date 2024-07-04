//! Defines Flags that the CPU contains

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    Carry    = 0b0000_0001, // Carry Flag
    Parity   = 0b0000_0010, // Parity Flag
    Zero     = 0b0000_1000, // Zero Flag
    Sign     = 0b0001_0000, // Sign Flag
    Overflow = 0b0010_0000, // Overflow Flag
}

impl Flag {
    // Function to return the flag value
    pub fn value(&self) -> u16 {
        *self as u16
    }
}
