pub struct VariableMetadata {
    pub start_index: usize,
    pub length: usize,
}

impl VariableMetadata {
    pub fn new(start_index: usize, length: usize) -> Self {
        return Self {start_index, length};
    }
}