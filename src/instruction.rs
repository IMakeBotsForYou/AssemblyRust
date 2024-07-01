pub enum Instruction {
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

impl Instruction {
    pub fn get_help_string(instruction: Instruction) -> String {
        match instruction {
            Instruction::Mov => {
                "The 'mov' instruction moves data between registers or between memory and registers.
Syntax:
    mov <reg>, <reg>
    mov <reg>, [<mem>]
    mov [<mem>], <reg>
    mov <reg>, <const>
    mov [<mem>], <const>
    mov <reg>, <var>".to_string()
            },
            Instruction::Push => {
                "The 'push' instruction pushes a value onto the stack.
Syntax:
    push <reg>
    push [<mem>]
    push <const>
    push <var>".to_string()
            },
            Instruction::Pop => {
                "The 'pop' instruction pops a value from the stack.
Syntax:
    pop <reg>
    pop [<mem>]".to_string()
            },
            Instruction::Lea => {
                "The 'lea' instruction loads the effective address of the operand into a register.
Syntax:
    lea <reg>, [<mem>]".to_string()
            },
            Instruction::Add => {
                "The 'add' instruction adds two operands.
Syntax:
    add <reg>, <reg>
    add <reg>, [<mem>]
    add <reg>, <const>
    add [<mem>], <reg>
    add [<mem>], <const>".to_string()
            },
            Instruction::Sub => {
                "The 'sub' instruction subtracts the second operand from the first.
Syntax:
    sub <reg>, <reg>
    sub <reg>, [<mem>]
    sub <reg>, <const>
    sub [<mem>], <reg>
    sub [<mem>], <const>".to_string()
            },
            Instruction::Inc => {
                "The 'inc' instruction increments an operand by one.
Syntax:
    inc <reg>
    inc [<mem>]".to_string()
            },
            Instruction::Dec => {
                "The 'dec' instruction decrements an operand by one.
Syntax:
    dec <reg>
    dec [<mem>]".to_string()
            },
            Instruction::Mul => {
                "The 'mul' instruction multiplies the operand by the accumulator.
Syntax:
    mul <reg>
    mul <b/w> [<mem>]
    mul <const>
    mul <var>".to_string()
            },
            Instruction::Imul => {
                "The 'imul' instruction multiplies the operand by the accumulator, using signed integer parsing.
Syntax:
    imul <reg>
    imul <b/w> [<mem>]
    imul <const>
    imul <var>".to_string()
            },
            Instruction::Div => {
                "The 'div' instruction divides the accumulator by the operand.
Syntax:
    div <reg>
    div <b/w> [<mem>]
    div <var>
    div <const>".to_string()
            },
            Instruction::Idiv => {
                "The 'idiv' instruction divides the accumulator by the operand.
Syntax:
    idiv <reg>
    idiv <b/w> [<mem>]
    idiv <var>
    idiv <const>".to_string()
            },
            Instruction::And => {
                "The 'and' instruction performs a bitwise AND operation.
Syntax:
    and <reg>, <reg>
    and <reg>, [<mem>]
    and <reg>, <const>
    and [<mem>], <reg>
    and [<mem>], <const>".to_string()
            },
            Instruction::Or => {
                "The 'or' instruction performs a bitwise OR operation.
Syntax:
    or <reg>, <reg>
    or <reg>, [<mem>]
    or <reg>, <const>
    or [<mem>], <reg>
    or [<mem>], <const>".to_string()
            },
            Instruction::Xor => {
                "The 'xor' instruction performs a bitwise exclusive OR operation.
Syntax:
    xor <reg>, <reg>
    xor <reg>, [<mem>]
    xor <reg>, <const>
    xor [<mem>], <reg>
    xor [<mem>], <const>".to_string()
            },
            Instruction::Not => {
                "The 'not' instruction performs a bitwise NOT operation.
Syntax:
    not <reg>
    not [<mem>]".to_string()
            },
            Instruction::Neg => {
                "The 'neg' instruction negates the operand, creating its two's complement.
Syntax:
    neg <reg>
    neg [<mem>]".to_string()
            },
            Instruction::Shl => {
                "The 'shl' instruction shifts the bits of the operand to the left.
Syntax:
    shl <reg>, <const>
    shl [<mem>], <const>
    shl <reg>, <cl>
    shl [<mem>], <cl>".to_string()
            },
            Instruction::Shr => {
                "The 'shr' instruction shifts the bits of the operand to the right.
Syntax:
    shr <reg>, <const>
    shr [<mem>], <const>
    shr <reg>, <cl>
    shr [<mem>], <cl>".to_string()
            },
            Instruction::Jmp => {
                "The 'jmp' instruction jumps to the specified label or memory location.
Syntax:
    jmp <label>
    jmp [<mem>]".to_string()
            },
            Instruction::Je => {
                "The 'je' instruction jumps to the specified label if the zero flag (ZF) is set, meaning the last comparison resulted in equality.
Syntax:
    je <label>
Explanation:
    Jump if the first operand is equal to the second operand.".to_string()
            },
            Instruction::Jne => {
                "The 'jne' instruction jumps to the specified label if the zero flag (ZF) is not set, meaning the last comparison did not result in equality.
Syntax:
    jne <label>
Explanation:
    Jump if the first operand is not equal to the second operand.".to_string()
            },
            Instruction::Jz => {
                "The 'jz' instruction jumps to the specified label if the zero flag (ZF) is set (alias for 'je').
Syntax:
    jz <label>
Explanation:
    Jump if the first operand is equal to the second operand.".to_string()
            },
            Instruction::Jnz => {
                "The 'jnz' instruction jumps to the specified label if the zero flag (ZF) is not set (alias for 'jne').
Syntax:
    jnz <label>
Explanation:
Jump if the first operand is not equal to the second operand.".to_string()
            },
            Instruction::Jg => {
                "The 'jg' instruction jumps to the specified label if the zero flag (ZF) is not set and the sign flag (SF) equals the overflow flag (OF), meaning the first operand is greater than the second operand in signed comparison.
Syntax:
    jg <label>
Explanation:
    Jump if the first operand is greater than the second operand (signed).".to_string()
            },
            Instruction::Jge => {
                "The 'jge' instruction jumps to the specified label if the sign flag (SF) equals the overflow flag (OF), meaning the first operand is greater than or equal to the second operand in signed comparison.
Syntax:
    jge <label>
Explanation:
    Jump if the first operand is greater than or equal to the second operand (signed).".to_string()
            },
            Instruction::Jl => {
                "The 'jl' instruction jumps to the specified label if the sign flag (SF) does not equal the overflow flag (OF), meaning the first operand is less than the second operand in signed comparison.
Syntax:
    jl <label>
Explanation:
    Jump if the first operand is less than the second operand (signed).".to_string()
            },
            Instruction::Jle => {
                "The 'jle' instruction jumps to the specified label if the zero flag (ZF) is set or the sign flag (SF) is not equal to the overflow flag (OF), meaning the first operand is less than or equal to the second operand in signed comparison.
Syntax:
    jle <label>
Explanation:
    Jump if the first operand is less than or equal to the second operand (signed).".to_string()
            },
            Instruction::Ja => {
                "The 'ja' instruction jumps to the specified label if the carry flag (CF) and the zero flag (ZF) are both not set, meaning the first operand is greater than the second operand in unsigned comparison.
Syntax:
    ja <label>
Explanation:
    Jump if the first operand is greater than the second operand (unsigned).".to_string()
            },
            Instruction::Jae => {
                "The 'jae' instruction jumps to the specified label if the carry flag (CF) is not set, meaning the first operand is greater than or equal to the second operand in unsigned comparison.
Syntax:
    jae <label>
Explanation:
    Jump if the first operand is greater than or equal to the second operand (unsigned).".to_string()
            },
            Instruction::Jb => {
                "The 'jb' instruction jumps to the specified label if the carry flag (CF) is set, meaning the first operand is less than the second operand in unsigned comparison.
Syntax:
    jb <label>
Explanation:
    Jump if the first operand is less than the second operand (unsigned).".to_string()
            },
            Instruction::Jbe => {
                "The 'jbe' instruction jumps to the specified label if the carry flag (CF) is set or the zero flag (ZF) is set, meaning the first operand is less than or equal to the second operand in unsigned comparison.
Syntax:
    jbe <label>
Explanation:
    Jump if the first operand is less than or equal to the second operand (unsigned).".to_string()
            },
            Instruction::Cmp => {
                "The 'cmp' instruction compares two operands.
Syntax:
    cmp <reg>, <reg>
    cmp <reg>, [<mem>]
    cmp <reg>, <const>
    cmp [<mem>], <reg>
    cmp [<mem>], <const>".to_string()
            },
            Instruction::Call => {
                "The 'call' instruction calls a procedure at the specified label or memory location.
Syntax:
    call <label>
    call [<mem>]".to_string()
            },
            Instruction::Ret => {
                "The 'ret' instruction returns from a procedure.
Syntax:
    ret
    ret <const>".to_string()
            },
        }
    }
}

// impl FromStr for Instruction {
//     type Err = ErrorCode;

//     fn from_str(s: &str) -> Result<Instruction, ErrorCode> {
//         match s.to_lowercase().as_str() {
//             "mov" => Ok(Instruction::Mov),
//             "push" => Ok(Instruction::Push),
//             "pop" => Ok(Instruction::Pop),
//             "add" => Ok(Instruction::Add),
//             "sub" => Ok(Instruction::Sub),
//             "lea" => Ok(Instruction::Lea),
//             "inc" => Ok(Instruction::Inc),
//             "dec" => Ok(Instruction::Dec),
//             "mul" => Ok(Instruction::Mul),
//             "div" => Ok(Instruction::Div),
//             "and" => Ok(Instruction::And),
//             "or" => Ok(Instruction::Or),
//             "xor" => Ok(Instruction::Xor),
//             "not" => Ok(Instruction::Not),
//             "neg" => Ok(Instruction::Neg),
//             "shl" => Ok(Instruction::Shl),
//             "shr" => Ok(Instruction::Shr),
//             "jmp" => Ok(Instruction::Jmp),
//             "je" => Ok(Instruction::Je),
//             "jne" => Ok(Instruction::Jne),
//             "jz" => Ok(Instruction::Jz),
//             "jnz" => Ok(Instruction::Jnz),
//             "jg" => Ok(Instruction::Jg),
//             "jge" => Ok(Instruction::Jge),
//             "jl" => Ok(Instruction::Jl),
//             "jle" => Ok(Instruction::Jle),
//             "cmp" => Ok(Instruction::Cmp),
//             "call" => Ok(Instruction::Call),
//             "ret" => Ok(Instruction::Ret),
//             _ => Err(ErrorCode::InvalidOpcode),
//         }
//     }
// }
