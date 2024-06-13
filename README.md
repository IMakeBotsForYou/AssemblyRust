# 8086 TASM Simulator with 32-bit Registers

This project is an 8086 TASM (Turbo Assembler) simulator that supports 32-bit registers. The simulator includes various assembly commands for performing operations such as data movement, arithmetic, stack manipulation, and conditional jumps.

## Features

- **32-bit Registers**: EAX, EBX, ECX, EDX, ESI, EDI
- **16-bit, 8-bit Registers**: AX (AL/AH), BX (BH/BL), etc.
- **Stack Operations**: `push` and `pop` commands. (Not yet implemented)
- **Arithmetic Operations**: Addition, subtraction, multiplication, and division.
- **Conditional and Unconditional Jumps**: For flow control.

## Supported Commands

### Mov
Move data between registers, memory, and constants.

Syntax:
mov <reg>, <reg>
mov <reg>, [<mem>]
mov [<mem>], <reg>
mov <reg>, <const>
mov [<mem>], <const>




### Push
Push data onto the stack.

Syntax:
push <reg>
push [<mem>]
push <const>

csharp


### Pop
Pop data from the stack.

Syntax:
pop <reg>
pop [<mem>]




### Lea
Load effective address.

Syntax:
lea <reg>, [<mem>]

sql


### Add
Add values to registers or memory.

Syntax:
add <reg>, <reg>
add <reg>, [<mem>]
add <reg>, <const>
add [<mem>], <reg>
add [<mem>], <const>

vbnet


### Sub
Subtract values from registers or memory.

Syntax:
sub <reg>, <reg>
sub <reg>, [<mem>]
sub <reg>, <const>
sub [<mem>], <reg>
sub [<mem>], <const>




### Inc
Increment a register or memory value.

Syntax:
inc <reg>
inc [<mem>]




### Dec
Decrement a register or memory value.

Syntax:
dec <reg>
dec [<mem>]




### Mul
Unsigned multiplication.

Syntax:
mul <reg>
mul [<mem>]
mul <const>
mul <var>




### Imul
Signed multiplication.

Syntax:
imul <reg>
imul [<mem>]
imul <const>
imul <var>




### Div
Unsigned division.

Syntax:
div <reg>
div [<mem>]
div <var>
div <const>




### Idiv
Signed division.

Syntax:
idiv <reg>
idiv [<mem>]
idiv <var>
idiv <const>




### Jmp
Unconditional jump.

Syntax:
jmp <label>




### Je
Jump if equal.

Syntax:
je <label>




### Jne
Jump if not equal.

Syntax:
jne <label>




### Jz
Jump if zero.

Syntax:
jz <label>




### Jnz
Jump if not zero.

Syntax:
jnz <label>




### Jg
Jump if greater.

Syntax:
jg <label>




### Jge
Jump if greater or equal.

Syntax:
jge <label>




### Jle
Jump if less or equal.

Syntax:
jle <label>




### Jl
Jump if less.

Syntax:
jl <label>




### Jb
Jump if below.

Syntax:
jb <label>




### Jbe
Jump if below or equal.

Syntax:
jbe <label>




### Ja
Jump if above.

Syntax:
ja <label>




### Jae
Jump if above or equal.

Syntax:
jae <label>




### Cmp
Compare two values.

Syntax:
cmp <reg>, <reg>
cmp <reg>, [<mem>]
cmp <reg>, <const>
cmp [<mem>], <reg>
cmp [<mem>], <const>

less


## Example Code

The following example generates a Fibonacci sequence.

```assembly
a dd 0
b dd 1
c dd 0

mov CX, 10

loop:
   mov EAX, [a]
   add EAX, [b]
   mov [c], EAX

   mov EAX, [b]
   mov [a], EAX
   mov EAX, [c]
   mov [b], EAX
   print [c]
   dec CX
   jnz loop


Output:

[SAVED] Saved variable a @ 0
[SAVED] Saved variable b @ 4
[SAVED] Saved variable c @ 8

[PRINT]@[IP=15] [c]: 1
[PRINT]@[IP=15] [c]: 2
[PRINT]@[IP=15] [c]: 3
[PRINT]@[IP=15] [c]: 5
[PRINT]@[IP=15] [c]: 8
[PRINT]@[IP=15] [c]: 13
[PRINT]@[IP=15] [c]: 21
[PRINT]@[IP=15] [c]: 34
[PRINT]@[IP=15] [c]: 55
[PRINT]@[IP=15] [c]: 89

Execution completed successfully.
Register EAX:   89      (00000000 01011001)
Register EBX:   0       (00000000 00000000)
Register ECX:   0       (00000000 00000000)
Register EDX:   0       (00000000 00000000)
Register ESI:   0       (00000000 00000000)
Register EDI:   0       (00000000 00000000)
Register IP:    17      (00000000 00010001)
Register FLAG:  10      (00000000 00001010)