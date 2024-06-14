pub enum Command {
    Mov,   
    /* Move data
    Syntax
    mov <reg>, <reg>
    mov <reg>, [<mem>]
    mov [<mem>], <reg>
    mov <reg>, <const>
    mov [<mem>], <const>
    */
    Push,
    /* Push to stack
    Syntax
    push <reg>
    push [<mem>]
    push <const>
    */
    Pop,
    /* Pop from stack
    Syntax
    pop <reg>
    pop [<mem>]
    */
    Lea,
    /* 
    Syntax
    lea <reg>, [<mem>]
    */
    Add,
    /*
    Syntax
    add <reg>, <reg>
    add <reg>, [<mem>]
    add <reg>, <const>
    add [<mem>], <reg>
    add [<mem>], <const>
    */
    Sub,
    /*
    Syntax
    sub <reg>, <reg>
    sub <reg>, [<mem>]
    sub <reg>, <const>
    sub [<mem>], <reg>
    sub [<mem>], <const>
    */
    Inc,
    /*
    Syntax
    inc <reg>
    inc [<mem>]
    */
    Dec,
    /*
    Syntax
    dec <reg>
    dec [<mem>]
    */
    Mul,
    /*
    Syntax
    mul <reg>
    mul [<mem>]
    mul <const>
    mul <var>
    */
    Imul,
    /*
    ditto
     */
    Div,
    /*
    Syntax
    div <reg>
    div [<mem>]
    div <var>
    div <const>
    */
    Idiv,
    /*
    ditto
     */
    And,
    /*
    Syntax
    and <reg>, <reg>
    and <reg>, [<mem>]
    and <reg>, <const>
    and [<mem>], <reg>
    and [<mem>], <const>
    */
    Or,
    /*
    Syntax
    or <reg>, <reg>
    or <reg>, [<mem>]
    or <reg>, <const>
    or [<mem>], <reg>
    or [<mem>], <const>
    */
    Xor,
    /*
    Syntax
    xor <reg>, <reg>
    xor <reg>, [<mem>]
    xor <reg>, <const>
    xor [<mem>], <reg>
    xor [<mem>], <const>
    */
    Not,
    /*
    Syntax
    not <reg>
    not [<mem>]
    */
    Neg,
    /*
    Syntax
    neg <reg>
    neg [<mem>]
    */
    Shl,
    /*
    Syntax
    shl <reg>, <const>
    shl [<mem>], <const>
    shl <reg>, <cl>
    shl [<mem>], <cl>
    */
    Shr,
    /*
    Syntax
    shr <reg>, <const>
    shr [<mem>], <const>
    shr <reg>, <cl>
    shr [<mem>], <cl>
    */
    Jmp,
    /*
    Syntax
    jmp <label>
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
    Jnz,
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
    Jl,
    /*
    Syntax
    jle <label>
    */
    Jb,
    /*
    Syntax
    jle <label>
    */
    Jbe,
    /*
    Syntax
    jle <label>
    */
    Ja,
    /*
    Syntax
    jle <label>
    */
    Jae,
    /*
    Syntax
    jle <label>
    */
    Cmp,
    /*
    Syntax
    cmp <reg>, <reg>
    cmp <reg>, [<mem>]
    cmp <reg>, <const>
    cmp [<mem>], <reg>
    cmp [<mem>], <const>
    */
    Call,
    /*
    Syntax
    call <label>
    call [<mem>]
    */
    Ret,
    /*
    Syntax
    ret
    ret <const>
    */
}

// #[derive(Debug)]

impl Command {
    pub fn get_help_string(command: Command) -> String {
        match command {
            Command::Mov => {
                "The 'mov' command moves data between registers or between memory and registers.
Syntax:
    mov <reg>, <reg>
    mov <reg>, [<mem>]
    mov [<mem>], <reg>
    mov <reg>, <const>
    mov [<mem>], <const>
    mov <reg>, <var>".to_string()
            },
            Command::Push => {
                "The 'push' command pushes a value onto the stack.
Syntax:
    push <reg>
    push [<mem>]
    push <const>
    push <var>".to_string()
            },
            Command::Pop => {
                "The 'pop' command pops a value from the stack.
Syntax:
    pop <reg>
    pop [<mem>]".to_string()
            },
            Command::Lea => {
                "The 'lea' command loads the effective address of the operand into a register.
Syntax:
    lea <reg>, [[<mem>]]".to_string()
            },
            Command::Add => {
                "The 'add' command adds two operands.
Syntax:
    add <reg>, <reg>
    add <reg>, [<mem>]
    add <reg>, <const>
    add [<mem>], <reg>
    add [<mem>], <const>".to_string()
            },
            Command::Sub => {
                "The 'sub' command subtracts the second operand from the first.
Syntax:
    sub <reg>, <reg>
    sub <reg>, [<mem>]
    sub <reg>, <const>
    sub [<mem>], <reg>
    sub [<mem>], <const>".to_string()
            },
            Command::Inc => {
                "The 'inc' command increments an operand by one.
Syntax:
    inc <reg>
    inc [<mem>]".to_string()
            },
            Command::Dec => {
                "The 'dec' command decrements an operand by one.
Syntax:
    dec <reg>
    dec [<mem>]".to_string()
            },
            Command::Mul => {
                "The 'mul' command multiplies the operand by the accumulator.
Syntax:
    mul <reg>
    mul <b/w> [<mem>]
    mul <const>
    mul <var>".to_string()
            },
            Command::Imul => {
                "The 'imul' command multiplies the operand by the accumulator, using signed integer parsing.
Syntax:
    imul <reg>
    imul <b/w> [<mem>]
    imul <const>
    imul <var>".to_string()
            },
            Command::Div => {
                "The 'div' command divides the accumulator by the operand.
Syntax:
    div <reg>
    div <b/w> [<mem>]
    div <var>
    div <const>".to_string()
            },
            Command::Idiv => {
                "The 'idiv' command divides the accumulator by the operand.
Syntax:
    idiv <reg>
    idiv <b/w> [<mem>]
    idiv <var>
    idiv <const>".to_string()
            },
            
            Command::And => {
                "The 'and' command performs a bitwise AND operation.
Syntax:
    and <reg>, <reg>
    and <reg>, [<mem>]
    and <reg>, <const>
    and [<mem>], <reg>
    and [<mem>], <const>".to_string()
            },
            Command::Or => {
                "The 'or' command performs a bitwise OR operation.
Syntax:
    or <reg>, <reg>
    or <reg>, [<mem>]
    or <reg>, <const>
    or [<mem>], <reg>
    or [<mem>], <const>".to_string()
            },
            Command::Xor => {
                "The 'xor' command performs a bitwise exclusive OR operation.
Syntax:
    xor <reg>, <reg>
    xor <reg>, [<mem>]
    xor <reg>, <const>
    xor [<mem>], <reg>
    xor [<mem>], <const>".to_string()
            },
            Command::Not => {
                "The 'not' command performs a bitwise NOT operation.
Syntax:
    not <reg>
    not [<mem>]".to_string()
            },
            Command::Neg => {
                "The 'neg' command negates the operand, creating its two's complement.
Syntax:
    neg <reg>
    neg [<mem>]".to_string()
            },
            Command::Shl => {
                "The 'shl' command shifts the bits of the operand to the left.
Syntax:
    shl <reg>, <const>
    shl [<mem>], <const>
    shl <reg>, <cl>
    shl [<mem>], <cl>".to_string()
            },
            Command::Shr => {
                "The 'shr' command shifts the bits of the operand to the right.
Syntax:
    shr <reg>, <const>
    shr [<mem>], <const>
    shr <reg>, <cl>
    shr [<mem>], <cl>".to_string()
            },
            Command::Jmp => {
                "The 'jmp' command jumps to the specified label or memory location.
Syntax:
    jmp <label>
    jmp [<mem>]".to_string()
            },
            Command::Je => {
                "The 'je' command jumps to the specified label if the zero flag (ZF) is set, meaning the last comparison resulted in equality.
Syntax:
    je <label>
Explanation:
    Jump if the first operand is equal to the second operand.".to_string()
            },
            Command::Jne => {
                "The 'jne' command jumps to the specified label if the zero flag (ZF) is not set, meaning the last comparison did not result in equality.
Syntax:
    jne <label>
Explanation:
    Jump if the first operand is not equal to the second operand.".to_string()
            },
            Command::Jz => {
                "The 'jz' command jumps to the specified label if the zero flag (ZF) is set (alias for 'je').
Syntax:
    jz <label>
Explanation:
    Jump if the first operand is equal to the second operand.".to_string()
            },
            Command::Jnz => {
                "The 'jnz' command jumps to the specified label if the zero flag (ZF) is not set (alias for 'jne').
Syntax:
    jnz <label>
Explanation:
Jump if the first operand is not equal to the second operand.".to_string()
            },
            Command::Jg => {
                "The 'jg' command jumps to the specified label if the zero flag (ZF) is not set and the sign flag (SF) equals the overflow flag (OF), meaning the first operand is greater than the second operand in signed comparison.
Syntax:
    jg <label>
Explanation:
    Jump if the first operand is greater than the second operand (signed).".to_string()
            },
            Command::Jge => {
                "The 'jge' command jumps to the specified label if the sign flag (SF) equals the overflow flag (OF), meaning the first operand is greater than or equal to the second operand in signed comparison.
Syntax:
    jge <label>
Explanation:
    Jump if the first operand is greater than or equal to the second operand (signed).".to_string()
            },
            Command::Jl => {
                "The 'jl' command jumps to the specified label if the sign flag (SF) does not equal the overflow flag (OF), meaning the first operand is less than the second operand in signed comparison.
Syntax:
    jl <label>
Explanation:
    Jump if the first operand is less than the second operand (signed).".to_string()
            },
            Command::Jle => {
                "The 'jle' command jumps to the specified label if the zero flag (ZF) is set or the sign flag (SF) is not equal to the overflow flag (OF), meaning the first operand is less than or equal to the second operand in signed comparison.
Syntax:
    jle <label>
Explanation:
    Jump if the first operand is less than or equal to the second operand (signed).".to_string()
            },
            Command::Ja => {
                "The 'ja' command jumps to the specified label if the carry flag (CF) and the zero flag (ZF) are both not set, meaning the first operand is greater than the second operand in unsigned comparison.
Syntax:
    ja <label>
Explanation:
    Jump if the first operand is greater than the second operand (unsigned).".to_string()
            },
            Command::Jae => {
                "The 'jae' command jumps to the specified label if the carry flag (CF) is not set, meaning the first operand is greater than or equal to the second operand in unsigned comparison.
Syntax:
    jae <label>
Explanation:
    Jump if the first operand is greater than or equal to the second operand (unsigned).".to_string()
            },
            Command::Jb => {
                "The 'jb' command jumps to the specified label if the carry flag (CF) is set, meaning the first operand is less than the second operand in unsigned comparison.
Syntax:
    jb <label>
Explanation:
    Jump if the first operand is less than the second operand (unsigned).".to_string()
            },
            Command::Jbe => {
                "The 'jbe' command jumps to the specified label if the carry flag (CF) is set or the zero flag (ZF) is set, meaning the first operand is less than or equal to the second operand in unsigned comparison.
Syntax:
    jbe <label>
Explanation:
    Jump if the first operand is less than or equal to the second operand (unsigned).".to_string()
            },
            Command::Cmp => {
                "The 'cmp' command compares two operands.
Syntax:
    cmp <reg>, <reg>
    cmp <reg>, [<mem>]
    cmp <reg>, <const>
    cmp [<mem>], <reg>
    cmp [<mem>], <const>".to_string()
            },
            Command::Call => {
                "The 'call' command calls a procedure at the specified label or memory location.
Syntax:
    call <label>
    call [<mem>]".to_string()
            },
            Command::Ret => {
                "The 'ret' command returns from a procedure.
Syntax:
    ret
    ret <const>".to_string()
            },
        }
    }
}

// impl FromStr for Command {
//     type Err = ErrorCode;

//     fn from_str(s: &str) -> Result<Command, ErrorCode> {
//         match s.to_lowercase().as_str() {
//             "mov" => Ok(Command::Mov),
//             "push" => Ok(Command::Push),
//             "pop" => Ok(Command::Pop),
//             "add" => Ok(Command::Add),
//             "sub" => Ok(Command::Sub),
//             "lea" => Ok(Command::Lea),
//             "inc" => Ok(Command::Inc),
//             "dec" => Ok(Command::Dec),
//             "mul" => Ok(Command::Mul),
//             "div" => Ok(Command::Div),
//             "and" => Ok(Command::And),
//             "or" => Ok(Command::Or),
//             "xor" => Ok(Command::Xor),
//             "not" => Ok(Command::Not),
//             "neg" => Ok(Command::Neg),
//             "shl" => Ok(Command::Shl),
//             "shr" => Ok(Command::Shr),
//             "jmp" => Ok(Command::Jmp),
//             "je" => Ok(Command::Je),
//             "jne" => Ok(Command::Jne),
//             "jz" => Ok(Command::Jz),
//             "jnz" => Ok(Command::Jnz),
//             "jg" => Ok(Command::Jg),
//             "jge" => Ok(Command::Jge),
//             "jl" => Ok(Command::Jl),
//             "jle" => Ok(Command::Jle),
//             "cmp" => Ok(Command::Cmp),
//             "call" => Ok(Command::Call),
//             "ret" => Ok(Command::Ret),
//             _ => Err(ErrorCode::InvalidOpcode),
//         }
//     }
// }