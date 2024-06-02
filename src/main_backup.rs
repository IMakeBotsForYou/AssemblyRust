use std::fs::File;
use std::io::{self, BufRead, BufReader};

struct Register {
    value: u8,
    name: String,
}

enum Command {
    Mov,   
    /*
    Syntax
    mov <reg>, <reg>
    mov <reg>, <mem>
    mov <mem>, <reg>
    mov <reg>, <const>
    mov <mem>, <const>
    */
    Push,
    /*
    Syntax
    push <reg>
    push <mem>
    push <const>
    */
    Pop,
    /*
    Syntax
    pop <reg>
    pop <mem>
    */
    Lea,
    /*
    Syntax
    lea <reg>, <mem>
    */
    Add,
    /*
    Syntax
    add <reg>, <reg>
    add <reg>, <mem>
    add <reg>, <const>
    add <mem>, <reg>
    add <mem>, <const>
    */
    Sub,
    /*
    Syntax
    sub <reg>, <reg>
    sub <reg>, <mem>
    sub <reg>, <const>
    sub <mem>, <reg>
    sub <mem>, <const>
    */
    Inc,
    /*
    Syntax
    inc <reg>
    inc <mem>
    */
    Dec,
    /*
    Syntax
    dec <reg>
    dec <mem>
    */
    Imul,
    /*
    Syntax
    imul <reg>
    imul <reg>, <reg>
    imul <reg>, <mem>
    imul <reg>, <const>
    imul <reg>, <reg>, <const>
    imul <reg>, <mem>, <const>
    */
    Idiv,
    /*
    Syntax
    idiv <reg>
    idiv <mem>
    */
    And,
    /*
    Syntax
    and <reg>, <reg>
    and <reg>, <mem>
    and <reg>, <const>
    and <mem>, <reg>
    and <mem>, <const>
    */
    Or,
    /*
    Syntax
    or <reg>, <reg>
    or <reg>, <mem>
    or <reg>, <const>
    or <mem>, <reg>
    or <mem>, <const>
    */
    Xor,
    /*
    Syntax
    xor <reg>, <reg>
    xor <reg>, <mem>
    xor <reg>, <const>
    xor <mem>, <reg>
    xor <mem>, <const>
    */
    Not,
    /*
    Syntax
    not <reg>
    not <mem>
    */
    Neg,
    /*
    Syntax
    neg <reg>
    neg <mem>
    */
    Shl,
    /*
    Syntax
    shl <reg>, <const>
    shl <mem>, <const>
    shl <reg>, <cl>
    shl <mem>, <cl>
    */
    Shr,
    /*
    Syntax
    shr <reg>, <const>
    shr <mem>, <const>
    shr <reg>, <cl>
    shr <mem>, <cl>
    */
    Jmp,
    /*
    Syntax
    jmp <label>
    jmp <mem>
    */
    Je,
    /*
    Syntax
    je <label>
    */
    Jne,
    /*
    Syntax
    jne <label>
    */
    Jz,
    /*
    Syntax
    jz <label>
    */
    Jg,
    /*
    Syntax
    jg <label>
    */
    Jge,
    /*
    Syntax
    jge <label>
    */
    Jle,
    /*
    Syntax
    jle <label>
    */
    Cmp,
    /*
    Syntax
    cmp <reg>, <reg>
    cmp <reg>, <mem>
    cmp <reg>, <const>
    cmp <mem>, <reg>
    cmp <mem>, <const>
    */
    Call,
    /*
    Syntax
    call <label>
    call <mem>
    */
    Ret,
    /*
    Syntax
    ret
    ret <const>
    */
}



impl Register {
    fn new(name: &str) -> Self {
        Self { value: 0, name: name.to_string() }
    }

    fn get_byte() {
        self.value
    }

    fn load(&mut self, value: u8) {
        self.value = value;
    }
}

struct Engine {
    lines: Vec<String>, // lines of source code (.txt)
    registers: [Register; 7], // A-D, ESI, EDI, P
    memory: [u8; 1024], // 1024 bytes of memory
    mode: bool, // false = reading data, true = reading code
    stack_pointer: u16, // pointer to the top of the stack within memory
    labels: Vec<u16>,
}

#[derive(Default)]
struct Register; // Assuming Register is defined elsewhere

struct Engine {
    lines: Vec<String>, // lines of source code (.txt)
    registers: [Register; 7], // A-D, ESI, EDI, P
    memory: [u8; 1024], // 1024 bytes of memory
    mode: bool, // false = reading data, true = reading code
    stack_pointer: usize, // pointer to the top of the stack within memory
    labels: Vec<usize>,
}

#[derive(Debug, PartialEq)]
enum Flag {
    Carry,
    Reserved,
    Parity,
    AuxiliaryCarry,
    Zero,
    Sign,
    Trap,
    InterruptEnable,
    Direction,
    Overflow,
}

impl Flag {
    fn to_string(&self) -> &'static str {
        match self {
            Flag::Carry => "Carry",
            Flag::Reserved => "Reserved",
            Flag::Parity => "Parity",
            Flag::AuxiliaryCarry => "Auxiliary Carry",
            Flag::Zero => "Zero",
            Flag::Sign => "Sign",
            Flag::Trap => "Trap",
            Flag::InterruptEnable => "Interrupt Enable",
            Flag::Direction => "Direction",
            Flag::Overflow => "Overflow",
        }
    }

    fn from_string(name: &str) -> Option<Self> {
        match name {
            "Carry" => Some(Flag::Carry),
            "Reserved" => Some(Flag::Reserved),
            "Parity" => Some(Flag::Parity),
            "Auxiliary Carry" => Some(Flag::AuxiliaryCarry),
            "Zero" => Some(Flag::Zero),
            "Sign" => Some(Flag::Sign),
            "Trap" => Some(Flag::Trap),
            "Interrupt Enable" => Some(Flag::InterruptEnable),
            "Direction" => Some(Flag::Direction),
            "Overflow" => Some(Flag::Overflow),
            _ => None,
        }
    }

    fn to_value(&self) -> u16 {
        match self {
            Flag::Carry => 0x0001,
            Flag::Reserved => 0x0002,
            Flag::Parity => 0x0004,
            Flag::AuxiliaryCarry => 0x0010,
            Flag::Zero => 0x0040,
            Flag::Sign => 0x0080,
            Flag::Trap => 0x0100,
            Flag::InterruptEnable => 0x0200,
            Flag::Direction => 0x0400,
            Flag::Overflow => 0x0800,
        }
    }

    fn from_value(value: u16) -> Option<Self> {
        match value {
            0x0001 => Some(Flag::Carry),
            0x0002 => Some(Flag::Reserved),
            0x0004 => Some(Flag::Parity),
            0x0010 => Some(Flag::AuxiliaryCarry),
            0x0040 => Some(Flag::Zero),
            0x0080 => Some(Flag::Sign),
            0x0100 => Some(Flag::Trap),
            0x0200 => Some(Flag::InterruptEnable),
            0x0400 => Some(Flag::Direction),
            0x0800 => Some(Flag::Overflow),
            _ => None,
        }
    }
}

impl Engine {
    fn new(file_name: &String) -> Self {
        read_lines_from_file(file_name);
        let my_registers: [Register; 8] = {
            Register::new("AX"),
            Register::new("BX"),
            Register::new("CX"),
            Register::new("DX"),
            Register::new("ESI"),
            Register::new("EDI"),
            Register::new("P"),
            Register::new("FLAG"),
        };
        Self {
            lines: Vec::new(),
            registers: my_registers,
            memory: [0; 1024],
            mode: false,
            stack_pointer: 1024, // Initialize stack pointer to the end of memory
            labels: Vec::new(),
            status: 
        }
    }
    
    fn turn_on_flag(name: String) {
        let value: u8 = Flag::to_value(Flag::from_string(name)?);
        let my_reg = self.get_register("FLAG".to_string());
        my_reg.value = my_reg.value | value; // OR
    }

    fn turn_off_flag(name: String){
        let value: u8 = Flag::to_value(Flag::from_string(name)?);
        let my_reg = self.get_register("FLAG".to_string());
        my_reg.value = my_reg.value ^ value; // XOR 
    }

    fn get_register(name: String) -> &mut Register{
        match name:
        "AX".to_string() => self.registers[0],
        "BX".to_string() => self.registers[1],
        "CX".to_string() => self.registers[2],
        "DX".to_string() => self.registers[3],
        "ESI".to_string() => self.registers[4],
        "DSI".to_string() => self.registers[5],
        "P".to_string() => self.registers[6],   
        "FLAG".to_string() => self.registers[7],   
        _ => panic!("Invalid register name"),
    }

    fn execute(&mut self) -> u8 {
        loop {
            // Read the current instruction pointer
            let ip = self.instruction_pointer;

            // Fetch the opcode from memory
            let opcode = self.memory[ip];

            // Increment the instruction pointer
            self.instruction_pointer += 1;

            match opcode {
                // MOV operations
                0x01 => {
                    // MOV_REG_REG (opcode 0x01)
                    let dest = self.memory[ip + 1] as usize;
                    let src = self.memory[ip + 2] as usize;
                    self.mov_reg_reg(dest, src);
                    self.instruction_pointer += 2;
                },
                0x02 => {
                    // MOV_REG_MEM (opcode 0x02)
                    let dest = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    self.mov_reg_mem(dest, addr);
                    self.instruction_pointer += std::mem::size_of::<usize>() - 1;
                },
                0x03 => {
                    // MOV_MEM_REG (opcode 0x03)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let src = self.memory[ip + 1 + std::mem::size_of::<usize>()] as usize;
                    self.mov_mem_reg(addr, src);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },
                0x04 => {
                    // MOV_REG_CONST (opcode 0x04)
                    let dest = self.memory[ip + 1] as usize;
                    let value = self.memory[ip + 2];
                    self.mov_reg_const(dest, value);
                    self.instruction_pointer += 2;
                },
                0x05 => {
                    // MOV_MEM_CONST (opcode 0x05)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let value = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.mov_mem_const(addr, value);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },

                // ADD operations
                0x10 => {
                    // ADD_REG_REG (opcode 0x10)
                    let dest = self.memory[ip + 1] as usize;
                    let src = self.memory[ip + 2] as usize;
                    self.add_reg_reg(dest, src);
                    self.instruction_pointer += 2;
                },
                0x11 => {
                    // ADD_REG_MEM (opcode 0x11)
                    let dest = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    self.add_reg_mem(dest, addr);
                    self.instruction_pointer += std::mem::size_of::<usize>() - 1;
                },
                0x12 => {
                    // ADD_REG_CONST (opcode 0x12)
                    let dest = self.memory[ip + 1] as usize;
                    let value = self.memory[ip + 2];
                    self.add_reg_const(dest, value);
                    self.instruction_pointer += 2;
                },
                0x13 => {
                    // ADD_MEM_REG (opcode 0x13)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let src = self.memory[ip + 1 + std::mem::size_of::<usize>()] as usize;
                    self.add_mem_reg(addr, src);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },
                0x14 => {
                    // ADD_MEM_CONST (opcode 0x14)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let value = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.add_mem_const(addr, value);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },

                // SUB operations
                0x20 => {
                    // SUB_REG_REG (opcode 0x20)
                    let dest = self.memory[ip + 1] as usize;
                    let src = self.memory[ip + 2] as usize;
                    self.sub_reg_reg(dest, src);
                    self.instruction_pointer += 2;
                },
                0x21 => {
                    // SUB_REG_MEM (opcode 0x21)
                    let dest = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    self.sub_reg_mem(dest, addr);
                    self.instruction_pointer += std::mem::size_of::<usize>() - 1;
                },
                0x22 => {
                    // SUB_REG_CONST (opcode 0x22)
                    let dest = self.memory[ip + 1] as usize;
                    let value = self.memory[ip + 2];
                    self.sub_reg_const(dest, value);
                    self.instruction_pointer += 2;
                },
                0x23 => {
                    // SUB_MEM_REG (opcode 0x23)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let src = self.memory[ip + 1 + std::mem::size_of::<usize>()] as usize;
                    self.sub_mem_reg(addr, src);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },
                0x24 => {
                    // SUB_MEM_CONST (opcode 0x24)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let value = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.sub_mem_const(addr, value);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },

                // INC operations
                0x30 => {
                    // INC_REG (opcode 0x30)
                    let reg = self.memory[ip + 1] as usize;
                    self.inc_reg(reg);
                    self.instruction_pointer += 1;
                },
                0x31 => {
                    // INC_MEM (opcode 0x31)
                    let addr = self.read_usize_from_memory(ip + 1);
                    self.inc_mem(addr);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },

                // DEC operations
                0x40 => {
                    // DEC_REG (opcode 0x40)
                    let reg = self.memory[ip + 1] as usize;
                    self.dec_reg(reg);
                    self.instruction_pointer += 1;
                },
                0x41 => {
                    // DEC_MEM (opcode 0x41)
                    let addr = self.read_usize_from_memory(ip + 1);
                    self.dec_mem(addr);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },

                // IMUL operations
                0x50 => {
                    // IMUL_REG (opcode 0x50)
                    let reg = self.memory[ip + 1] as usize;
                    self.imul_reg(reg);
                    self.instruction_pointer += 1;
                },
                0x51 => {
                    // IMUL_REG_REG (opcode 0x51)
                    let dest = self.memory[ip + 1] as usize;
                    let src = self.memory[ip + 2] as usize;
                    self.imul_reg_reg(dest, src);
                    self.instruction_pointer += 2;
                },
                0x52 => {
                    // IMUL_REG_MEM (opcode 0x52)
                    let dest = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    self.imul_reg_mem(dest, addr);
                    self.instruction_pointer += std::mem::size_of::<usize>() - 1;
                },
                0x53 => {
                    // IMUL_REG_CONST (opcode 0x53)
                    let dest = self.memory[ip + 1] as usize;
                    let value = self.memory[ip + 2];
                    self.imul_reg_const(dest, value);
                    self.instruction_pointer += 2;
                },
                0x54 => {
                    // IMUL_REG_REG_CONST (opcode 0x54)
                    let dest = self.memory[ip + 1] as usize;
                    let src = self.memory[ip + 2] as usize;
                    let value = self.memory[ip + 3];
                    self.imul_reg_reg_const(dest, src, value);
                    self.instruction_pointer += 3;
                },
                0x55 => {
                    // IMUL_REG_MEM_CONST (opcode 0x55)
                    let dest = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    let value = self.memory[ip + 2 + std::mem::size_of::<usize>()];
                    self.imul_reg_mem_const(dest, addr, value);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },

                // IDIV operations
                0x60 => {
                    // IDIV_REG (opcode 0x60)
                    let reg = self.memory[ip + 1] as usize;
                    self.idiv_reg(reg);
                    self.instruction_pointer += 1;
                },
                0x61 => {
                    // IDIV_MEM (opcode 0x61)
                    let addr = self.read_usize_from_memory(ip + 1);
                    self.idiv_mem(addr);
                }
                                // AND operations
                0x70 => {
                    // AND_REG_REG (opcode 0x70)
                    let dest = self.memory[ip + 1] as usize;
                    let src = self.memory[ip + 2] as usize;
                    self.and_reg_reg(dest, src);
                    self.instruction_pointer += 2;
                },
                0x71 => {
                    // AND_REG_MEM (opcode 0x71)
                    let dest = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    self.and_reg_mem(dest, addr);
                    self.instruction_pointer += std::mem::size_of::<usize>() - 1;
                },
                0x72 => {
                    // AND_REG_CONST (opcode 0x72)
                    let dest = self.memory[ip + 1] as usize;
                    let value = self.memory[ip + 2];
                    self.and_reg_const(dest, value);
                    self.instruction_pointer += 2;
                },
                0x73 => {
                    // AND_MEM_REG (opcode 0x73)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let src = self.memory[ip + 1 + std::mem::size_of::<usize>()] as usize;
                    self.and_mem_reg(addr, src);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },
                0x74 => {
                    // AND_MEM_CONST (opcode 0x74)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let value = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.and_mem_const(addr, value);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },

                // OR operations
                0x80 => {
                    // OR_REG_REG (opcode 0x80)
                    let dest = self.memory[ip + 1] as usize;
                    let src = self.memory[ip + 2] as usize;
                    self.or_reg_reg(dest, src);
                    self.instruction_pointer += 2;
                },
                0x81 => {
                    // OR_REG_MEM (opcode 0x81)
                    let dest = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    self.or_reg_mem(dest, addr);
                    self.instruction_pointer += std::mem::size_of::<usize>() - 1;
                },
                0x82 => {
                    // OR_REG_CONST (opcode 0x82)
                    let dest = self.memory[ip + 1] as usize;
                    let value = self.memory[ip + 2];
                    self.or_reg_const(dest, value);
                    self.instruction_pointer += 2;
                },
                0x83 => {
                    // OR_MEM_REG (opcode 0x83)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let src = self.memory[ip + 1 + std::mem::size_of::<usize>()] as usize;
                    self.or_mem_reg(addr, src);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },
                0x84 => {
                    // OR_MEM_CONST (opcode 0x84)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let value = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.or_mem_const(addr, value);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },

                // XOR operations
                0x90 => {
                    // XOR_REG_REG (opcode 0x90)
                    let dest = self.memory[ip + 1] as usize;
                    let src = self.memory[ip + 2] as usize;
                    self.xor_reg_reg(dest, src);
                    self.instruction_pointer += 2;
                },
                0x91 => {
                    // XOR_REG_MEM (opcode 0x91)
                    let dest = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    self.xor_reg_mem(dest, addr);
                    self.instruction_pointer += std::mem::size_of::<usize>() - 1;
                },
                0x92 => {
                    // XOR_REG_CONST (opcode 0x92)
                    let dest = self.memory[ip + 1] as usize;
                    let value = self.memory[ip + 2];
                    self.xor_reg_const(dest, value);
                    self.instruction_pointer += 2;
                },
                0x93 => {
                    // XOR_MEM_REG (opcode 0x93)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let src = self.memory[ip + 1 + std::mem::size_of::<usize>()] as usize;
                    self.xor_mem_reg(addr, src);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },
                0x94 => {
                    // XOR_MEM_CONST (opcode 0x94)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let value = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.xor_mem_const(addr, value);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },

                // NOT operations
                0xA0 => {
                    // NOT_REG (opcode 0xA0)
                    let reg = self.memory[ip + 1] as usize;
                    self.not_reg(reg);
                    self.instruction_pointer += 1;
                },
                0xA1 => {
                    // NOT_MEM (opcode 0xA1)
                    let addr = self.read_usize_from_memory(ip + 1);
                    self.not_mem(addr);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },

                // NEG operations
                0xB0 => {
                    // NEG_REG (opcode 0xB0)
                    let reg = self.memory[ip + 1] as usize;
                    self.neg_reg(reg);
                    self.instruction_pointer += 1;
                },
                0xB1 => {
                    // NEG_MEM (opcode 0xB1)
                    let addr = self.read_usize_from_memory(ip + 1);
                    self.neg_mem(addr);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },

                // SHL operations
                0xC0 => {
                    // SHL_REG_CONST (opcode 0xC0)
                    let reg = self.memory[ip + 1] as usize;
                    let count = self.memory[ip + 2];
                    self.shl_reg_const(reg, count);
                    self.instruction_pointer += 2;
                },
                0xC1 => {
                    // SHL_MEM_CONST (opcode 0xC1)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let count = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.shl_mem_const(addr, count);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },
                0xC2 => {
                    // SHL_REG_CL (opcode 0xC2)
                    let reg = self.memory[ip + 1] as usize;
                    self.shl_reg_cl(reg);
                    self.instruction_pointer += 1;
                },
                0xC3 => {
                    // SHL_MEM_CL (opcode 0xC3)
                    let addr = self.read_usize_from_memory(ip + 1);
                    self.shl_mem_cl(addr);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },

                // SHR operations
                0xD0 => {
                    // SHR_REG_CONST (opcode 0xD0)
                    let reg = self.memory[ip + 1] as usize;
                    let count = self.memory[ip + 2];
                    self.shr_reg_const(reg, count);
                    self.instruction_pointer += 2;
                },
                0xD1 => {
                    // SHR_MEM_CONST (opcode 0xD1)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let count = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.shr_mem_const(addr, count);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },
                0xD2 => {
                    // SHR_REG_CL (opcode 0xD2)
                    let reg = self.memory[ip + 1] as usize;
                    self.shr_reg_cl(reg);
                    self.instruction_pointer += 1;
                },
                0xD3 => {
                    // SHR_MEM_CL (opcode 0xD3)
                    let addr = self.read_usize_from_memory(ip + 1);
                    self.shr_mem_cl(addr);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },
                // JMP operations
                0xE0 => {
                    // JMP (opcode 0xE0)
                    let address = self.read_usize_from_memory(ip + 1);
                    self.jmp(address);
                    // No need to increment instruction pointer as jmp changes it directly
                },

                // CMP operations
                0xF0 => {
                    // CMP_REG_REG (opcode 0xF0)
                    let reg1 = self.memory[ip + 1] as usize;
                    let reg2 = self.memory[ip + 2] as usize;
                    self.cmp_reg_reg(reg1, reg2);
                    self.instruction_pointer += 2;
                },
                0xF1 => {
                    // CMP_REG_MEM (opcode 0xF1)
                    let reg = self.memory[ip + 1] as usize;
                    let addr = self.read_usize_from_memory(ip + 2);
                    self.cmp_reg_mem(reg, addr);
                    self.instruction_pointer += std::mem::size_of::<usize>() - 1;
                },
                0xF2 => {
                    // CMP_REG_CONST (opcode 0xF2)
                    let reg = self.memory[ip + 1] as usize;
                    let value = self.memory[ip + 2];
                    self.cmp_reg_const(reg, value);
                    self.instruction_pointer += 2;
                },
                0xF3 => {
                    // CMP_MEM_REG (opcode 0xF3)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let reg = self.memory[ip + 1 + std::mem::size_of::<usize>()] as usize;
                    self.cmp_mem_reg(addr, reg);
                    self.instruction_pointer += std::mem::size_of::<usize>();
                },
                0xF4 => {
                    // CMP_MEM_CONST (opcode 0xF4)
                    let addr = self.read_usize_from_memory(ip + 1);
                    let value = self.memory[ip + 1 + std::mem::size_of::<usize>()];
                    self.cmp_mem_const(addr, value);
                    self.instruction_pointer += std::mem::size_of::<usize>() + 1;
                },

                // JE/JZ operations
                0x100 => {
                    // JE (opcode 0x100)
                    let address = self.read_usize_from_memory(ip + 1);
                    self.je(address);
                    // No need to increment instruction pointer as je changes it directly
                },

                // JNE/JNZ operations
                0x101 => {
                    // JNE (opcode 0x101)
                    let address = self.read_usize_from_memory(ip + 1);
                    self.jne(address);
                    // No need to increment instruction pointer as jne changes it directly
                },

                // CALL operations
                0x110 => {
                    // CALL (opcode 0x110)
                    let address = self.read_usize_from_memory(ip + 1);
                    self.call(address);
                    // No need to increment instruction pointer as call changes it directly
                },

                // RET operations
                0x120 => {
                    // RET (opcode 0x120)
                    self.ret();
                    // No need to increment instruction pointer as ret changes it directly
                },

                // HALT operations
                0x130 => {
                    // HALT (opcode 0x130)
                    self.halt();
                    return self.status;
                },

                // Default case for unrecognized opcodes
                _ => {
                    // Unrecognized opcode
                    self.status = Status::Error(ErrorCode::InvalidOpcode);
                    return self.status;
                }
            }    
        }
    }

    fn check_result_flags(result: usize) {
        let flags: u16 = 0x0000;
        if result == 0:
            flags |= Flag::to_value(Flag::Zero);

    }

    fn halt() {
        self.status = Status::Halted;
    }
    // Stack operations
    fn push(&mut self, value: u8) {
        if self.stack_pointer == 1024 - 256 {
            panic!("Stack overflow");
        }
        self.stack_pointer -= 1;
        self.memory[self.stack_pointer] = value;
    }

    fn pop(&mut self) -> u8 {
        if self.stack_pointer == 1024 {
            panic!("Stack underflow");
        }
        let value = self.memory[self.stack_pointer];
        self.stack_pointer += 1;
        value
    }

    // MOV operations
    fn mov_reg_reg(&mut self, dest: String, src: usize) {
        self.get_register(dest).value = self.registers[src].clone();
    }

    fn mov_reg_mem(&mut self, dest: String addr: usize) {
        // Assuming Register has a method to load from a byte
        self.get_register(dest).load(self.memory[addr]);
    }

    fn mov_mem_reg(&mut self, addr: usize, src: String) {
        // Assuming Register has a method to get as a byte
        self.memory[addr] = self.get_register(src).value.get_byte();
    }

    fn mov_reg_const(&mut self, dest: usize, value: u8) {
        // Assuming Register has a method to load from a byte
        self.get_register(dest).load(value);
    }

    fn mov_mem_const(&mut self, addr: usize, value: u8) {
        self.memory[addr] = value;
    }

    // ADD operation
    fn add_reg_reg(&mut self, src: String, dest: String) {
        // Assuming Register has a method to get and set as byte
        let result = self.get_register(dest).get_byte().wrapping_add(self.get_register(src).get_byte());
        self.get_register(dest).load(result);
    }

    fn add_reg_mem(&mut self, dest: String, addr: usize) {
        let result = self.get_register(dest).get_byte().wrapping_add(self.memory[addr]);
        self.get_register(dest).load(result);
    }

    fn add_reg_const(&mut self, dest: String, value: u8) {
        let result = self.get_register(dest).get_byte().wrapping_add(value);
        self.get_register(dest).load(result);
    }

    fn add_mem_reg(&mut self, addr: usize, src: String) {
        let result = self.memory[addr].wrapping_add(self.get_register(src).get_byte());
        self.memory[addr] = result;
    }

    fn add_mem_const(&mut self, addr: usize, value: u8) {
        let result = self.memory[addr].wrapping_add(value);
        self.memory[addr] = result;
    }

    // SUB operation
    fn sub_reg_reg(&mut self, dest: String, src: String) {
        let result = self.get_register(dest).get_byte().wrapping_sub(self.get_register(src).get_byte());
        self.get_register(dest).load(result);
    }

    fn sub_reg_mem(&mut self, dest: String, addr: usize) {
        let result = self.get_register(dest).get_byte().wrapping_sub(self.memory[addr]);
        self.get_register(dest).load(result);
    }

    fn sub_reg_const(&mut self, dest: usize, value: u8) {
        let result = self.get_register(dest)get_byte().wrapping_sub(value);
        self.get_register(dest).load(result);
    }

    fn sub_mem_reg(&mut self, addr: usize, src: String) {
        let result = self.memory[addr].wrapping_sub(self.get_register(src).get_byte());
        self.memory[addr] = result;
    }

    fn sub_mem_const(&mut self, addr: usize, value: u8) {
        let result = self.memory[addr].wrapping_sub(value);
        self.memory[addr] = result;
    }

    // INC operation
    fn inc_reg(&mut self, reg: String) {
        let result = self.get_register(reg).get_byte().wrapping_add(1);
        self.get_register(reg).load(result);
    }

    fn inc_mem(&mut self, addr: usize) {
        let result = self.memory[addr].wrapping_add(1);
        self.memory[addr] = result;
    }

    // DEC operation
    fn dec_reg(&mut self, reg: String) {
        let result = self.get_register(reg).get_byte().wrapping_sub(1);
        self.get_register(reg).load(result);
    }

    fn dec_mem(&mut self, addr: usize) {
        let result = self.memory[addr].wrapping_sub(1);
        self.memory[addr] = result;
    }

    // IMUL operation
    fn imul_reg(&mut self, reg: String) {
        let result = self.get_register(reg).get_byte().wrapping_mul(self.get_register(reg).get_byte());
        self.get_register(reg).load(result);
    }

    fn imul_reg_reg(&mut self, dest: String, src: String) {
        let result = self.get_register(dest).get_byte().wrapping_mul(self.get_register(src).get_byte());
        self.get_register(dest).load(result);
    }

    fn imul_reg_mem(&mut self, dest: String, addr: usize) {
        let result = self.get_register(dest).get_byte().wrapping_mul(self.memory[addr]);
        self.get_register(dest).load(result);
    }

    fn imul_reg_const(&mut self, dest: String, value: u8) {
        let result = self.get_register(dest).get_byte().wrapping_mul(value);
        self.get_register(dest).load(result);
    }

    fn imul_reg_reg_const(&mut self, dest: usize, src: String, value: u8) {
        let result = self.get_register(dest).get_byte().wrapping_mul(self.get_register(dest).get_byte()).wrapping_mul(value);
        self.get_register(dest).load(result);
    }

    fn imul_reg_mem_const(&mut self, dest: usize, addr: usize, value: u8) {
        let result = self.get_register(dest).get_byte().wrapping_mul(self.memory[addr]).wrapping_mul(value);
        self.get_register(dest).load(result);
    }

    fn idiv_reg(&mut self, reg: usize) {
        let divisor = self.get_register(reg).value as u16;

        if self.get_register("DX").value == 0 {
            panic!("Division by zero");
        }

        let edx_eax = ((self.get_register("DX").value as u32) << 16) | (self.get_register("AX").value as u32);
        let quotient = (edx_eax / (divisor as u32)) as u16;
        let remainder = (edx_eax % (divisor as u32)) as u16;

        self.get_register("AX").load(quotient as u8);
        self.get_register("DX").load(remainder as u8);
    }

    fn idiv_mem(&mut self, addr: usize) {
        let divisor = self.memory[addr] as u16;

        if self.get_register("DX").value == 0 {
            panic!("Division by zero");
        }

        let edx_eax = ((self.get_register("DX").value as u32) << 16) | (self.get_register("AX").value as u32);
        let quotient = (edx_eax / (divisor as u32)) as u16;
        let remainder = (edx_eax % (divisor as u32)) as u16;

        self.get_register("AX").load(quotient as u8);
        self.get_register("DX").load(remainder as u8);
    }

    // AND operation
    fn and_reg_reg(&mut self, dest: usize, src: usize) {
        let result = self.registers[dest].get_byte() & self.registers[src].get_byte();
        self.registers[dest].load(result);
    }

    fn and_reg_mem(&mut self, dest: usize, addr: usize) {
        let result = self.registers[dest].get_byte() & self.memory[addr];
        self.registers[dest].load(result);
    }

    fn and_reg_const(&mut self, dest: usize, value: u8) {
        let result = self.registers[dest].get_byte() & value;
        self.registers[dest].load(result);
    }

    fn and_mem_reg(&mut self, addr: usize, src: usize) {
        let result = self.memory[addr] & self.registers[src].get_byte();
        self.memory[addr] = result;
    }

    fn and_mem_const(&mut self, addr: usize, value: u8) {
        let result = self.memory[addr] & value;
        self.memory[addr] = result;
    }

    // OR operation
    fn or_reg_reg(&mut self, dest: usize, src: usize) {
        let result = self.registers[dest].get_byte() | self.registers[src].get_byte();
        self.registers[dest].load(result);
    }

    fn or_reg_mem(&mut self, dest: usize, addr: usize) {
        let result = self.registers[dest].get_byte() | self.memory[addr];
        self.registers[dest].load(result);
    }

    fn or_reg_const(&mut self, dest: usize, value: u8) {
        let result = self.registers[dest].get_byte() | value;
        self.registers[dest].load(result);
    }

    fn or_mem_reg(&mut self, addr: usize, src: usize) {
        let result = self.memory[addr] | self.registers[src].get_byte();
        self.memory[addr] = result;
    }

    fn or_mem_const(&mut self, addr: usize, value: u8) {
        let result = self.memory[addr] | value;
        self.memory[addr] = result;
    }

    // XOR operation
    fn xor_reg_reg(&mut self, dest: usize, src: usize) {
        let result = self.registers[dest].get_byte() ^ self.registers[src].get_byte();
        self.registers[dest].load(result);
    }

    fn xor_reg_mem(&mut self, dest: usize, addr: usize) {
        let result = self.registers[dest].get_byte() ^ self.memory[addr];
        self.registers[dest].load(result);
    }

    fn xor_reg_const(&mut self, dest: usize, value: u8) {
        let result = self.registers[dest].get_byte() ^ value;
        self.registers[dest].load(result);
    }

    fn xor_mem_reg(&mut self, addr: usize, src: usize) {
        let result = self.memory[addr] ^ self.registers[src].get_byte();
        self.memory[addr] = result;
    }

    fn xor_mem_const(&mut self, addr: usize, value: u8) {
        let result = self.memory[addr] ^ value;
        self.memory[addr] = result;
    }

    // NOT operation
    fn not_reg(&mut self, reg: usize) {
        let result = !self.registers[reg].get_byte();
        self.registers[reg].load(result);
    }

    fn not_mem(&mut self, addr: usize) {
        let result = !self.memory[addr];
        self.memory[addr] = result;
    }

    // NEG operation
    fn neg_reg(&mut self, reg: usize) {
        let result = self.registers[reg].get_byte().wrapping_neg();
        self.registers[reg].load(result);
    }

    fn neg_mem(&mut self, addr: usize) {
        let result = self.memory[addr].wrapping_neg();
        self.memory[addr] = result;
    }

    // SHL operation
    fn shl_reg_const(&mut self, reg: usize, count: u8) {
        let result = self.registers[reg].get_byte() << count;
        self.registers[reg].load(result);
    }

    fn shl_mem_const(&mut self, addr: usize, count: u8) {
        let result = self.memory[addr] << count;
        self.memory[addr] = result;
    }

    fn shl_reg_cl(&mut self, reg: usize) {
        let count = self.registers[1].get_byte() & 0x1F; // Assuming CL is register 1
        let result = self.registers[reg].get_byte() << count;
        self.registers[reg].load(result);
    }

    fn shl_mem_cl(&mut self, addr: usize) {
        let count = self.registers[1].get_byte() & 0x1F; // Assuming CL is register 1
        let result = self.memory[addr] << count;
        self.memory[addr] = result;
    }

    // SHR operation
    fn shr_reg_const(&mut self, reg: usize, count: u8) {
        let result = self.registers[reg].get_byte() >> count;
        self.registers[reg].load(result);
    }

    fn shr_mem_const(&mut self, addr: usize, count: u8) {
        let result = self.memory[addr] >> count;
        self.memory[addr] = result;
    }

    fn shr_reg_cl(&mut self, reg: usize) {
        let count = self.registers[1].get_byte() & 0x1F; // Assuming CL is register 1
        let result = self.registers[reg].get_byte() >> count;
        self.registers[reg].load(result);
    }

    fn shr_mem_cl(&mut self, addr: usize) {
        let count = self.registers[1].get_byte() & 0x1F; // Assuming CL is register 1
        let result = self.memory[addr] >> count;
        self.memory[addr] = result;
    }

    // JMP operation
    fn jmp(&mut self, address: usize) {
        // Assuming you have a way to change the instruction pointer
        self.instruction_pointer = address;
    }

    // JE operation
    fn je(&mut self, address: usize, condition: bool) {
        if condition {
            self.jmp(address);
        }
    }

    // JNE operation
    fn jne(&mut self, address: usize, condition: bool) {
        if !condition {
            self.jmp(address);
        }
    }

    // JZ operation
    fn jz(&mut self, address: usize, zero_flag: bool) {
        if zero_flag {
            self.jmp(address);
        }
    }

    // JG operation
    fn jg(&mut self, address: usize, greater_flag: bool) {
        if greater_flag {
            self.jmp(address);
        }
    }

    // JGE operation
    fn jge(&mut self, address: usize, greater_equal_flag: bool) {
        if greater_equal_flag {
            self.jmp(address);
        }
    }

    // JLE operation
    fn jle(&mut self, address: usize, less_equal_flag: bool) {
        if less_equal_flag {
            self.jmp(address);
        }
    }

    // CMP operation
    fn cmp_reg_reg(&mut self, reg1: usize, reg2: usize) {
        let value1 = self.get_register(reg1).get_byte();
        let value2 = self.get_register(dest).get_byte();
        self.update_flags(value1, value2);
    }

    fn cmp_reg_mem(&mut self, reg: String, addr: usize) {
        let value1 = self.get_register(dest).get_byte();
        let value2 = self.memory[addr];
        self.update_flags(value1, value2);
    }

    fn cmp_reg_const(&mut self, reg: String, value: u8) {
        let value1 = self.get_register(dest).get_byte();
        self.update_flags(value1, value);
    }

    fn cmp_mem_reg(&mut self, addr: usize, reg: String) {
        let value1 = self.memory[addr];
        let value2 = self.get_register(dest).get_byte();
        self.update_flags(value1, value2);
    }

    fn cmp_mem_const(&mut self, addr: usize, value: u8) {
        let value1 = self.memory[addr];
        self.update_flags(value1, value);
    }

    fn update_flags(&mut self, value1: u8, value2: u8) {
        // Implement flag updates here
    }

    // CALL operation
    fn call(&mut self, address: usize) {
        // Assuming you have a way to push the current instruction pointer
        self.push_instruction_pointer();
        self.jmp(address);
    }

    // RET operation
    fn ret(&mut self) {
        // Assuming you have a way to pop the instruction pointer
        self.pop_instruction_pointer();
    }

    fn push_instruction_pointer(&mut self, instruction_pointer: usize) {
        // Push instruction pointer to stack
        self.stack_pointer -= std::mem::size_of::<usize>();
        self.write_usize_to_memory(self.stack_pointer, instruction_pointer);
    }

    fn pop_instruction_pointer(&mut self) -> usize {
        // Pop instruction pointer from stack
        let instruction_pointer = self.read_usize_from_memory(self.stack_pointer);
        self.stack_pointer += std::mem::size_of::<usize>();
        instruction_pointer
    }

    fn write_usize_to_memory(&mut self, address: usize, value: usize) {
        // Write usize to memory at specified address
        let bytes = value.to_le_bytes();
        self.memory[address..address + std::mem::size_of::<usize>()].copy_from_slice(&bytes);
    }

    fn read_usize_from_memory(&self, address: usize) -> usize {
        // Read usize from memory at specified address
        let mut bytes = [0; std::mem::size_of::<usize>()];
        bytes.copy_from_slice(&self.memory[address..address + std::mem::size_of::<usize>()]);
        usize::from_le_bytes(bytes)
    }
}


fn read_lines_from_file(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}



#[derive(Debug)]
pub enum ErrorCode {
    DivisionByZero,
    StackOverflow,
    StackUnderflow,
    InvalidOpcode,
    // Add more error codes as needed
}

#[derive(Debug)]
pub enum Status {
    Ok,
    Error(ErrorCode),
    Halted,
}


fn main() -> io::Result<()> {
    let assembly = Engine::new("code.txt")?;

    // Print out the lines to verify
    for line in &assembly.lines {
        println!("{}", line);
    }

    // Optionally, print out the registers to verify
    for register in &assembly.registers {
        println!("Register {}: {}", register.name, register.value);
    }

    Ok(())
}
