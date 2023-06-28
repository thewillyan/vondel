# Registers

The assembler supports 25 registers. These registers can be used for general-purpose operations or not.

| Register | Suggested Usage | A Bus | B Bus | C Bus  |
| -------- | --------------- | ----- | ----- | ------ |
| Mar      | Memory Address  |       |       | Output |
| Mdr      | Memory Data     | Input | Input | Output |
| Mbr      | Memory Buffer   | Input |       |        |
| Mbru     | Memory Buffer   | Input |       |        |
| Mbr2     | Memory Buffer   | Input |       |        |
| Mbr2u    | Memory Buffer   | Input |       |        |
| Pc       | Program Counter | Input |       | Output |
| Cpp      | Call Pointer    | Input | Input | Output |
| Lv       | Local Variable  | Input | Input | Output |
| Ra       | Return Address  | Input | Input | Output |
| T0       | Temporary       | Input | Input | Output |
| T1       | Temporary       | Input | Input | Output |
| T2       | Temporary       | Input | Input | Output |
| T3       | Temporary       | Input | Input | Output |
| S0       | Saved           | Input | Input | Output |
| S1       | Saved           | Input | Input | Output |
| S2       | Saved           | Input | Input | Output |
| S3       | Saved           | Input | Input | Output |
| S4       | Saved           | Input | Input | Output |
| S5       | Saved           | Input | Input | Output |
| S6       | Saved           | Input | Input | Output |
| A0       | Function Arg    | Input | Input | Output |
| A1       | Function Arg    | Input | Input | Output |
| A2       | Function Arg    | Input | Input | Output |
| A3       | Function Arg    | Input | Input | Output |

# Sections

The assembler supports two sections: `.text` and `.data`. The sections are defined as follows:

## .text Section

The `.text` section contains the executable instructions.

## .data Section

The `.data` section is used for declaring and initializing data.

# Instruction Format

Most vondel instructions in follow the format: `opcode dest_regs <- source1, source2`. Here's a breakdown of the components:

- `opcode`: Specifies the operation to be performed.
- `dest_regs`: The registers where the result will be stored.
- `source1` and `source2`: Registers or immediate values used as operands.

# Supported Instructions

The assembler supports the following instructions:

- Arithmetic and Logic
  - [ADD](#add)
  - [SUB](#sub)
  - [MUL](#mul)
  - [MUL2](#mul2)
  - [DIV](#div)
  - [MOD](#mod)
  - [AND](#and)
  - [OR](#or)
  - [XOR](#xor)
  - [NOT](#not)
- Shift
  - [SLL (Shift Left Logical)](#sll)
  - [SLA (Shift Left Arithmetic)](#sla)
  - [SRA (Shift Right Arithmetic)](#sra)
- Memory
  - [READ](#read)
  - [WRITE](#write)
- Branch
  - [JAL](#jal)
  - [BEQ](#beq)
  - [BNE](#bne)
  - [BLT](#blt)
  - [BGT](#bgt)
- Immediate
  - [LUI](#lui)
  - [ADDI](#addi)
  - [MULI](#muli)
  - [DIVI](#divi)
  - [MODI](#modi)
  - [SUBI](#subi)
  - [ANDI](#andi)
  - [ORI](#ori)
  - [XORI](#xori)
- Other
  - [MOV](#mov)
  - [HALT](#halt)

## Add

Adds two registers and save the result into 1 up to 20 registers that are allowed in the C_bus

```
add t0, t1, s0, a2, ra <- a0, a1
```

## Sub

Subtract `x - y` and store on registers

```
sub t0, t1, s0, a2, ra <- a0, a1
```

## Mul

Multiplies the value of `x` and `y` and store on registers

```
mul s0, a2, ra <- a0, a1
```

> `WARNING`: Mul cannot be used with t\* registers

## Mul2

Multiplies the value of `x` and `y` and store on registers

```
mul s0, a2, ra <- a0, a1
```

> `INFO`: More performatic and can use t\* registers

## Div

Divides `x` by `y` and store on registers

```
div t0, t1, s0, a2, ra <- a0, a1
```

## Mod

Gets the remainder of `x` by `y` and store on registers

```
mod t0, t1, s0, a2, ra <- a0, a1
```

## And

Makes a bitwise `and` with `x` and `y`

```
and t0, t1, s0, a2, ra <- a0, a1
```

## Or

Makes a bitwise `or` with `x` and `y`

```
or t0, t1, t2, s0, s1, a2, ra <- a0, a1
```

## Xor

Makes a bitwise `xor` with `x` and `y`

```
xor t0, t1, t2, s0, s1, a2, ra <- a0, a1
```

## Not

Stores the result of `!x` on the registers

```
not t0, t1, t2, s0, s1, a2, ra <- a0
```

> It uses only a _**single**_ register as source

## Sll

Stores the result of `x << 8` on the registers

```
sll t0, t1 <- a0
```

> It uses only a _**single**_ register as source

## Sla

Stores the result of `x << 1` on the registers

```
sla t0, t1 <- a0
```

> It uses only a _**single**_ register as source

## Sra

Stores the result of `x >> 1` on the registers

```
sra t0, t1 <- a0
```

> It uses only a _**single**_ register as source

## Read

Load a value from memory on the address `addr` into a register

```
read t0, t1, t2 <- addr
read t0, t1, t2 <- 77
```

> Addr can be both a label referencing a variable in `.data` section or a immediate with value of 0 to 255

## Write

Store a value from a register `x` into memory on address `addr`

```
write addr <- t1
write 77 <- ra
```

> It allows only a _**single**_ register as source

> Addr can be both a label referencing a variable in `.data` section or a immediate with value of 0 to 255

## Jal

Jump inconditionally to a `label`

```
jal loop
```

## Beq

Branch if equal (jump to a `label` if `x` and `y` are equal)

```
beq t0, t1, done
```

## Bne

Branch if **not** equal (jump to a `label` if `x` and `y` are not equal)

```
bne t0, t1, done
```

## Blt

Branch if less then (jump to a `label` if `x` is less then `y`)

```
blt t0, t1, done
```

## Bgt

Branch if greater then (jump to a `label` if `x` is greater then `y`)

```
bgt t0, t1, done
```

## Lui

Load upper immediate `imm` on registers

```
lui t0, t1,t2 <- imm
lui t0, t1,t2 <- 77
```

> It takes only a single immediate as parameter

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Addi

Adds a register `x` and a immediate `imm` and save this value on registers

```
addi t0, t1,t2 <- t0, imm
addi t0, t1,t2 <- a2, 77
```

> The first argument must be a register and the second a immediate

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Muli

Multiplies a register `x` and an immediate `imm` and save this value on registers

```
muli t0, t1,t2 <- t0, imm
muli t0, t1,t2 <- a2, 77
```

> The first argument must be a register and the second an immediate

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Divi

Divides a register `x` by an immediate `imm` and save this value on registers

```
divi t0, t1,t2 <- t0, imm
divi t0, t1,t2 <- a2, 77
```

> The first argument must be a register and the second an immediate

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Modi

Get the remainder of a `div` between a register `x` by an immediate `imm` and save this value on registers

```
modi t0, t1,t2 <- t0, imm
modi t0, t1,t2 <- a2, 77
```

> The first argument must be a register and the second an immediate

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Subi

Subtracts from register `x` the value of an immediate `imm` and save this value on registers

```
subi t0, t1 <- t0, imm
subi t0, t1 <- a2, 77
```

> The first argument must be a register and the second an immediate

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Andi

Makes a bitwise `and` of register `x` and an immediate `imm`, then saves this value on registers

```
andi t0, t1 <- t0, imm
andi t0, t1 <- a2, 77
```

> The first argument must be a register and the second an immediate

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Ori

Makes a bitwise `or` of register `x` and an immediate `imm`, then saves this value on registers

```
ori t0, t1 <- t0, imm
ori t0, t1 <- a2, 77
```

> The first argument must be a register and the second an immediate

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Xori

Makes a bitwise `xor` of register `x` and an immediate `imm`, then saves this value on registers

```
xori t0, t1 <- t0, imm
xori t0, t1 <- a2, 77
```

> The first argument must be a register and the second an immediate

> Imm can be both a label referencing a `.byte` in `.data` section or an immediate with value of 0 to 255

## Mov

Store the value of the register x on the registers

```
mov t0, t1, t2 <- a1
mov ra, s1, s2, s3 <- t0
```

> It supports only a single register as parameter

## Halt

Stop the program execution

```
halt
```

> Just halt

## Data Declaration Instructions (`.data` section)

- `.byte`: Declare a byte-sized data item
- `.word`: Declare a word-sized (4 bytes) data item

# Labels and Branching

Labels can be defined and used as targets for branching instructions. Here are the guidelines for labels and branching:

- Labels are defined by placing a colon (`:`) after the label name (e.g., `label:`).
- Branching instructions can use labels as targets (e.g., `beq t1, t2, label`).

# Comments

Comments start with a semicolon (`;`) or (`#`) and continue until the end of the line. Comments are ignored during assembly.

# Addressing Modes

The assembler supports two addressing modes: immediate and register addressing. Here's how they are used:

- Immediate values can be specified directly in the instruction.
- Registers have the ABI form of RISC V

This specification provides a general outline for the assembler language. Additional details, such as specific opcode mappings and assembly directives, can be added as needed.
