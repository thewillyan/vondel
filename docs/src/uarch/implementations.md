# Instruction Implementations

In this section we will show how some of the microinstruction of the Vondel
Language are implemented in the microprogram.

## Operations Summary

- Arithmetic
  - [ADD](#add)
  - [SUB](#sub)
  - [MUL](#mul)
- Logic
  - [AND](#and)
  - [OR](#or)
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
  - [SUBI](#subi)
  - [ANDI](#andi)
  - [ORI](#ori)
- Other
  - [MOV](#mov)
  - [HALT](#halt)

## HALT

_HALT_ is a special microinstruction to indicate that
the program has ended. In this design this opcode is achieved by seting all 64
bits of the microinstruction to 1.

This constant is avalible as a shortcut in the `CtrlStore` struct.

## MOV

Syntax: `mov r0, ..., rn <- x`.

Action: Store the value of the register `x` on the registers `r0, ..., rn`.

### High Level

```
mov r15, r13 <- r14
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011000|00000000000000000101|000|01110|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## ADD

Syntax: `add r0, ..., rn <- x, y`.

Action: Stores the result of `x + y` on the registers `r0, ..., rn`.

### High Level

```
add r0, r1 <- r2, r3
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |        C BUS       |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00111100|00001100000000000000|000|01011|01100|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## SUB

Syntax: `sub r0, ..., rn <- x, y`.

Action: Stores the result of `x - y` on the registers `r0, ..., rn`.

### High Level

```
sub r0 <- r2, r3
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00111111|00001000000000000000|000|01100|00110|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## AND
Syntax: `and r0, ..., rn <- x, y`.

Action: Stores the result of `x & y ` on the registers `r0, ..., rn`.

### High Level

```
and r1 <- r2, r3
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00001100|00000100000000000000|000|01011|01100|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## OR
Syntax: `or r0, ..., rn <- x, y`.

Action: Stores the result of `x | y ` on the registers `r0, ..., rn`.

### High Level

```
or r0, r1, r14, r15 <- r2, r3
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011100|00001100000000000011|000|01011|01100|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## NOT

Syntax: `not r0, ..., rn <- x`.

Action: Stores the result of `!x` on the registers `r0, ..., rn`.

### High Level

```
not r0, r15 <- r3
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011010|00001000000000000001|000|01100|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## SLL
Syntax: `sll r0, ..., rn <- x`.

Action: Stores the result of `x << 8` on the registers `r0, ..., rn`.

### High Level

```
sll r0, r15 <- r3
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|10011000|00001000000000000001|000|01100|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## SLA
Syntax: `sla r0, ..., rn <- x`.

Action: Stores the result of `x << 1` on the registers `r0, ..., rn`.

### High Level

```
sla r0, r15 <- r3
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|11011000|00001000000000000001|000|01100|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## SRA
Syntax: `sra r0, ..., rn <- x`.

Action: Stores the result of `x >> 1` on the registers `r0, ..., rn`.

### High Level

```
sra r0, r15 <- r3
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|01011000|00001000000000000001|000|01100|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## READ

Syntax: `read r0, ..., rn <- [addr]`.

Action: Stores the memory word at `addr` on the registers `r0, ..., rn`.

### High Level

```
read r0, r15 <- 0x00008
```

### Microprogram

Precondition: `0x00008` should be stored at MAR.

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00000000|00000000000000000000|010|11111|11111|  00000000 |
|1 |000000010|000|00011000|00001000000000000001|000|00000|11111|  00000000 |
|2 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## WRITE

Syntax: `write [addr] <- x`.

Action: Stores the memory value of `x` at `addr` on the memory.

### High Level

```
write 0x00001 <- r14
```

### Microprogram

Precondition: `0x00001` should be store at MAR.

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011000|10000000000000000000|000|01110|11111|  00000000 |
|1 |000000010|000|00000000|00000000000000000001|100|11111|11111|  00000000 |
|2 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## JAL
Syntax: `jal [addr]`.

Action: Jumps to the instruction of id (address) `addr`.

### High Level

```
jal 0x0A
```

### Microprogram

`JMPC` bit is used to nconditional branch, so we have to find some way of
storing `0x0A` into MBR and then use `JMPC` to jump to that address.
For that the number 10 (`0x0A`) must be stored at memory and its address
should be stored in PC. So, let's store 10 in the current address of the PC,
by default the PC is 0, so let's store it there.


|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00000000|00000000000000000000|010|11111|11111|  00000000 |
|1 |000000010|100|00000000|00000000000000000000|000|11111|11111|  00000000 |
|- |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|10|111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

In a nutshell the operations are the following:

1. Fetch the memory.
2. Set `JMPC` to 1.

## BEQ

Syntax: `beq x, y`.

Action: Jump if the value of the register `x` is equal to the value of the
register `y`.

### High Level

```
beq r14, r15
```

### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|001|00111111|00000000000000000000|000|10111|10011|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## BNE

Syntax: `bne x, y`.

Action: Jump if the value of the register `x` is **not** equal to the value of the
register `y`.

### High Level

```
bne r14, r15
```

### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|010|00111111|00000000000000000000|000|10111|10011|  00000000 |
|1   |000000010|010|00111111|00000000000000000000|000|11000|10010|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |
|258 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## BLT

Syntax: `blt x, y`.

Action: Jump if the value of the register `x` is less than the value of the
register `y`.

### High Level

```
blt r14, r15
```

### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|010|00111111|00000000000000000000|000|11000|10010|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## BGT

Syntax: `bgt x, y`.

Action: Jump if the value of the register `x` is greater than the value of the
register `y`.

### High Level

```
bgt r14, r15
```

### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|010|00111111|00000000000000000000|000|10111|10011|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |


## MUL
Syntax: `mul r0, ..., rn <- x, y`.

Action: Multiply the value of x by y .

### High Level

```
mul r13 <- r6, r7
```

### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|010|00111111|00000000000000000000|000|10000|01010|  00000000 |
|1   |000000010|000|00011000|00001000000000000000|000|10000|11111|  00000000 |
|2   |000000011|000|00111111|00000100000000000100|000|01111|11111|  00000000 |
|3   |000000100|001|00110110|00001000000000000000|000|01001|11111|  00000000 |
|4   |000000011|000|00111100|00000000000000000100|000|10110|00101|  00000000 |
|5   |000000011|000|00111111|00000100000000000100|000|10000|11111|  00000000 |
|6   |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |000000101|000|00011000|00001000000000000000|000|01111|11111|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|260 |000000110|000|00000000|00000000000000000000|000|00000|00000|  00000000 |

In a nutshell the operations are the following:

1. Jump to `0b100000001` (257) if `r6 - r7` is negative.
2. Case 1 (has not jumped), therefore `r6 >= r7`
    1. Store the value of `r7` on `r0`
    2. Store the value of `r6` on `r1` and `r13`
    3. Store the value of `r0 - 1` into `r0` or jump to the `TERMINATE` code (260) if equals to 0.
    4. Stores the value of `r13 + r1` into `r13`.
3. Case 2 (has jumped), therefore `r6 < r7`
    1. Store the value of `r6` on `r0`
    2. Store the value of `r7` on `r1` and `r13` and goto the step 3 of the case 1.

In another words, the smallest number is stored at `r0` and the greater is stored
in `r1` and we aways make `r0 * r1`. This optimization is necessary because a
multiplication is computed as sequential additions, and making less additions
save some clock cycles.

In the real assembly implementation the registers used to store the temporary
values are `T0` (to store the minimum), `T1` (to store the sum) and `T2` (to store the maximum),
so be caution when using this registers at the same time as the `mul` operation.

## MUL2

Syntax: `mul2 r0, ..., rn <- x, y`.

Action: Multiply the value of x by y .

The difference between `mul2` and `mul` are that `mul` is a software addition,
in other words, a sequential sum of a number, on the other hand `mul2` is
hardware implemented (uses a multiplication circuit).

### High Level

```
mul2 r13 <- r6, r7
```

### Microprogram

## LUI

Syntax: `lui r0, ..., rn <- [byte]`.

Action: Store the the immediate value of `byte` on the registers `r0, ..., rn`.

### High Level

```
lui r15 <- 0xFF
```

This program stores `0xFF` (255) to r15, but 255 is not stored in any register (it is
a _immediate value_).

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011000|00000000000000000001|000|01000|11111|  11111111 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## ADDI

Syntax: `addi r0, ..., rn <- x, [byte]`.

Action: Stores the value of `x + [byte]` on the registers `r0, ..., rn`.

### High Level

```
addi r14 <- r14, 0x05
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00111100|00000000000000000010|000|01000|10010|  00000101 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## SUBI

Syntax: `subi r0, ..., rn <- x, [byte]`.

Action: Stores the value of `x - [byte]` on the registers `r0, ..., rn`.

### High Level

```
subi r14 <- r14, 0x05
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00111111|00000000000000000010|000|01000|10010|  00000101 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## ANDI

Syntax: `andi r0, ..., rn <- x, [byte]`.

Action: Stores the value of `x & [byte]` on the registers `r0, ..., rn`.

### High Level

```
andi r14 <- r14, 0xFF
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00001100|00000000000000000010|000|01000|10010|  11111111 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## ORI

Syntax: `ori r0, ..., rn <- x, [byte]`.

Action: Stores the value of `x | [byte]` on the registers `r0, ..., rn`.

### High Level

```
ori r14 <- r14, 0x00
```

### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011100|00000000000000000010|000|01000|10010|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |
