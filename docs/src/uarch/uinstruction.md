# Microinstruction

Microinstructions are stored in the control store and are fetched with the address
stode at MPC (Microprogram Counter).

A Vondel microinstruction has 64 bits the following format:

|   NEXT  |  JAM   |   ALU  |  C BUS  |  MEM   |   A    |   B    | IMMEDIATE |
|:-------:|:------:|:------:|:-------:|:------:|:------:|:------:|:---------:|
| 9 bits  | 3 bits | 8 bits | 20 bits | 3 bits | 5 bits | 5 bits |  8 bits   |

You can find a more detailed version of this diagram [here](https://i.imgur.com/tlHAPgL.png).

## NEXT

The `NEXT` field stores the next microinstruction address. In other words, the
`NEXT` field stores the next value to be stored at MPC.

## JAM

The `JAM` field stores in which conditions the program should jump. A Jump is
a change in the natural flow of the microprogram that represents that the next
instruction should be changed is some way.

We have 3 possible condition to jump: if the value returned by ALU is 0 (JAMZ),
if is negative (JAMN) and if we want to change the next instruction using
something stored at MBR (JMPC). The first bit (MSB) from `JAM` is JMPC, the
second is JAMN and the last is JAMZ.

On JAMN and JAMZ a jump is, in fact, a OR operation on the most sigficant bit
of MPC and the condition. In other words, if the condition is true, which happens
when its bit is 1, that 1 is bitwise ORed with the MSB of MPC, so if the value
of MPC is `000001010` the jump position is `100001010`. But on JMPC,
a jump is a bitwise or with MBR and the 8 LSB's from MPC.

## ALU

The `ALU` field actually controls 2 devices: the ALU itself and the shifter
connected to its autput.

The ALU has two input, A and B (that come from the A and B bus, respectively) and 
it's controled by the 6 LSB's from the ALU field, they are:

- F0 and F1 (Controls the ALU function)
- ENA and ENB (Enables the input from A and B bus respectively)
- INVA (Inverts the bits of A)
- INC (Increments 1 to the ALU result)

from MSB to LSB. The logic and arithmetic functions that ALU can
operate are managed by F0 and F1 like this:

| F0 | F1 | Function |
|:--:|:--:|:---------|
| 0  | 0  | A AND B  |
| 0  | 1  | A OR B   |
| 1  | 0  | NOT B    |
| 1  | 1  | A + B    |

Some useful combinations of ALU signal can be found below:

| F0 | F1 | ENA | ENB | INVA | INC | Function |
|:--:|:--:|:---:|:---:|:----:|:---:|:---------|
| 0 | 1 | 1 | 0 | 0 | 0 | A |
| 0 | 1 | 0 | 1 | 0 | 0 | B |
| 0 | 1 | 1 | 0 | 1 | 0 | not A |
| 1 | 0 | 1 | 1 | 0 | 0 | not B |
| 1 | 1 | 1 | 1 | 0 | 0 | A + B |
| 1 | 1 | 1 | 1 | 0 | 1 | A + B + 1 |
| 1 | 1 | 1 | 0 | 0 | 1 | A + 1 |
| 1 | 1 | 0 | 1 | 0 | 1 | B + 1 |
| 1 | 1 | 1 | 1 | 1 | 1 | B − A |
| 1 | 1 | 0 | 1 | 1 | 0 | B − 1 |
| 1 | 1 | 1 | 0 | 1 | 1 | −A |
| 0 | 0 | 1 | 1 | 0 | 0 | A AND B |
| 0 | 1 | 1 | 1 | 0 | 0 | A OR B |
| 0 | 1 | 0 | 0 | 0 | 0 | 0 |
| 1 | 1 | 0 | 0 | 0 | 1 | 1 |
| 1 | 1 | 0 | 0 | 1 | 0 | −1 |

The shifter has 2 inputs: The value of the ALU operation (let's call it `X`) and
the operation opcode that are which are the 2 MSB's from the ALU field. The
operations and its opcode are as following:

| Opcode | Operation         | Output   |
| ------ | ----------------- | -------- |
| `0b00` | None              | `X`      |
| `0b01` | Shift Right 1 bit | `X >> 1` |
| `0b10` | Shift Left 8 bits | `X << 8` |
| `0b11` | Shift Left 1 bit  | `X << 1` |

## C BUS

The `C BUS` field represents which registers gonna be writen with the value
of the C BUS (which is the shifter output). This field has 20 bits because
there are 20 registers connected to the C bus, each bit 1 represents that the
register represented by that bit should be writen by the C BUS.

The relation between a bit n and which register it's represents is shown below
(from MSB to LSB):

| Bit | Register |
| --- | -------- | 
| 1   | MDR      |
| 2   | MAR      |
| 3   | PC       |
| 4   | LV       |
| 5   | R0       |
| 6   | R1       |
| 7   | R2       |
| 8   | R3       |
| 9   | R4       |
| 10  | R5       |
| 11  | R6       |
| 12  | R7       |
| 13  | R8       |
| 14  | R9       |
| 15  | R10      |
| 16  | R11      |
| 17  | R12      |
| 18  | R13      |
| 19  | R14      |
| 20  | R15      |

## MEM

The memory field represents which memory operations gonna happen in the cycle.
The bit 1 (MSB), 2 and 3 represents the operations of WRITE, READ and FETCH
respectively.

Each bit 1 in the field informs that the memory operation related to that
field will be executed.

## A and B

`A` and `B` are fields that control which register writes to the A and B bus
respectively, but, since the A and B bus are connected directly to the A and B
inputs of the ALU, they can also be viewed as which register goes as a entry
to the ALU operation.

The values of `A` and `B` and which register they enable (NONE,
represents that none of them writes to the respective BUS, so the value of the
bus is 0) is shown below:

Output to BUS A:

| ID | BIN | Register |
| -- | --- | -------- |
| 0  |00000| MDR      |
| 1  |00001| PC       |
| 2  |00010| MBR      |
| 3  |00011| MBRU     |
| 4  |00100| MBR2     |
| 5  |00101| MBR2U    |
| 6  |00110| LV       |
| 7  |00111| CPP      |
| 8  |01000| IMMEDIATE|
| 9  |01001| R0       |
| 10 |01010| R1       |
| 11 |01011| R2       |
| 12 |01100| R3       |
| 13 |01101| R4       |
| 14 |01110| R5       |
| 15 |01111| R6       |
| 16 |10000| R7       |
| 17 |10001| R8       |
| 18 |10010| R9       |
| 19 |10011| R10      |
| 20 |10100| R11      |
| 21 |10101| R12      |
| 22 |10110| R13      |
| 23 |10111| R14      |
| 24 |11000| R15      |
| .. | ... | NONE     |

---

Output to BUS B:

| ID | BIN | Register |
| -- | --- | -------- |
| 0  |00000| MDR      |
| 1  |00001| LV       |
| 2  |00010| CPP      |
| 3  |00011| IMMEDIATE|
| 4  |00100| R0       |
| 5  |00101| R1       |
| 6  |00110| R2       |
| 7  |00111| R3       |
| 8  |01000| R4       |
| 9  |01001| R5       |
| 10 |01010| R6       |
| 11 |01011| R7       |
| 12 |01100| R8       |
| 13 |01101| R9       |
| 14 |01110| R10      |
| 15 |01111| R11      |
| 16 |10000| R12      |
| 17 |10001| R13      |
| 18 |10010| R14      |
| 19 |10011| R15      |
| .. | ... | NONE     |

## IMMEDIATE

The immediate field allows us to send a arbitrary 8 bit number to the A or B
bus, i.e if we set `0x08` in the IMMEDIATE field and enable the immediate input
on the A and/or B bus, that `0x08` gonna be loaded in the corresponding bus.

## Instruction Implementations

In this section we will show how some of the microinstruction of the Vondel
Language are implemented in the microprogram.

### Operations Summary

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

### ADD

Syntax: `add x, y -> r0, ..., rn`.

Action: Stores the result of `x + y` on the registers `r0, ..., rn`.

#### High Level

```
add r2, r3 -> r0, r1
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |        C BUS       |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00111100|00001100000000000000|000|01011|01100|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### SUB

Syntax: `sub x, y -> r0, ..., rn`.

Action: Stores the result of `x - y` on the registers `r0, ..., rn`.

#### High Level

```
sub r2, r3 -> r0
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00111111|00001000000000000000|000|01100|00110|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### AND
Syntax: `and x, y -> r0, ..., rn`.

Action: Stores the result of `x & y ` on the registers `r0, ..., rn`.

#### High Level

```
and r2, r3 -> r1
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00001100|00000100000000000000|000|01011|01100|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### OR
Syntax: `or x, y -> r0, ..., rn`.

Action: Stores the result of `x | y ` on the registers `r0, ..., rn`.

#### High Level

```
or r2, r3 -> r0, r1, r14, r15
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011100|00001100000000000011|000|01011|01100|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### NOT

Syntax: `not x -> r0, ..., rn`.

Action: Stores the result of `!x` on the registers `r0, ..., rn`.

#### High Level

```
not r3 -> r0, r15
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011010|00001000000000000001|000|01100|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### SLL
Syntax: `sll x -> r0, ..., rn`.

Action: Stores the result of `x << 8` on the registers `r0, ..., rn`.

#### High Level

```
sll r3 -> r0, r15
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|10011000|00001000000000000001|000|01100|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### SLA
Syntax: `sla x -> r0, ..., rn`.

Action: Stores the result of `x << 1` on the registers `r0, ..., rn`.

#### High Level

```
sla r3 -> r0, r15
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|11011000|00001000000000000001|000|01100|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### SRA
Syntax: `sra x -> r0, ..., rn`.

Action: Stores the result of `x >> 1` on the registers `r0, ..., rn`.

#### High Level

```
sra r3 -> r0, r15
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|01011000|00001000000000000001|000|01100|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### READ

Syntax: `read [addr] -> r0, ..., rn`.

Action: Stores the memory word at `addr` on the registers `r0, ..., rn`.

#### High Level

```
read 0x00008 -> r0, r15
```

#### Microprogram

Precondition: `0x00008` should be stored at MAR.

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00000000|00000000000000000000|010|11111|11111|  00000000 |
|1 |000000010|000|00011000|00001000000000000001|000|00000|11111|  00000000 |
|2 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### WRITE

Syntax: `write x -> [addr]`.

Action: Stores the memory value of `x` at `addr` on the memory.

#### High Level

```
write r14 -> 0x00001
```

#### Microprogram

Precondition: `0x00001` should be store at MAR.

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011000|10000000000000000000|000|01110|11111|  00000000 |
|1 |000000010|000|00000000|00000000000000000001|100|11111|11111|  00000000 |
|2 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### JAL
Syntax: `jal [addr]`.

Action: Jumps to the instruction of id (address) `addr`.

#### High Level

```
jal 0x0A
```

#### Microprogram

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

### BEQ

Syntax: `beq x, y`.

Action: Jump if the value of the register `x` is equal to the value of the
register `y`.

#### High Level

```
beq r14, r15
```

#### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|001|00111111|00000000000000000000|000|10111|10011|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### BNE

Syntax: `bne x, y`.

Action: Jump if the value of the register `x` is **not** equal to the value of the
register `y`.

#### High Level

```
bne r14, r15
```

#### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|010|00111111|00000000000000000000|000|10111|10011|  00000000 |
|1   |000000010|010|00111111|00000000000000000000|000|11000|10010|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |
|258 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### BLT

Syntax: `blt x, y`.

Action: Jump if the value of the register `x` is less than the value of the
register `y`.

#### High Level

```
blt r14, r15
```

#### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|010|00111111|00000000000000000000|000|11000|10010|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### BGT

Syntax: `bgt x, y`.

Action: Jump if the value of the register `x` is greater than the value of the
register `y`.

#### High Level

```
bgt r14, r15
```

#### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|010|00111111|00000000000000000000|000|10111|10011|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |


### MUL
Syntax: `mul x, y -> r0, ..., rn`.

Action: Multiply the value of x by y .

#### High Level

```
mul r6, r7 -> r13
```

#### Microprogram

|ID  |   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-  |:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0   |000000001|010|00111111|00000000000000000000|000|10000|01010|  00000000 |
|1   |000000010|000|00011000|00001000000000000000|000|10000|11111|  00000000 |
|2   |000000011|000|00111111|00000100000000000100|000|01111|11111|  00000000 |
|3   |000000100|001|00110110|00001000000000000000|000|01001|11111|  00000000 |
|4   |000000011|000|00111100|00000000000000000100|000|10110|00101|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|257 |100000010|000|00011000|00001000000000000000|000|01111|11111|  00000000 |
|258 |000000011|000|00111111|00000100000000000100|000|10000|11111|  00000000 |
|-   |    -    | - |    -   |         -          | - |  -  |  -  |     -     |
|260 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

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

### LUI

Syntax: `lui [byte] -> r0, ..., rn`.

Action: Store the the immediate value of `byte` on the registers `r0, ..., rn`.

#### High Level

```
lui 0xFF -> r15
```

This program stores `0xFF` (255) to r15, but 255 is not stored in any register (it is
a _immediate value_).

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011000|00000000000000000001|000|01000|11111|  11111111 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### ADDI

Syntax: `addi x, [byte] -> r0, ..., rn`.

Action: Stores the value of `x + [byte]` on the registers `r0, ..., rn`.

#### High Level

```
addi r14, 0x05 -> r14
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00111100|00000000000000000010|000|01000|10010|  00000101 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### SUBI

Syntax: `subi x, [byte] -> r0, ..., rn`.

Action: Stores the value of `x - [byte]` on the registers `r0, ..., rn`.

#### High Level

```
subi r14, 0x05 -> r14
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00111111|00000000000000000010|000|01000|10010|  00000101 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### ANDI

Syntax: `andi x, [byte] -> r0, ..., rn`.

Action: Stores the value of `x & [byte]` on the registers `r0, ..., rn`.

#### High Level

```
andi r14, 0xFF -> r14
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00001100|00000000000000000010|000|01000|10010|  11111111 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### ORI

Syntax: `ori x, [byte] -> r0, ..., rn`.

Action: Stores the value of `x | [byte]` on the registers `r0, ..., rn`.

#### High Level

```
ori r14, 0x00 -> r14
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011100|00000000000000000010|000|01000|10010|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

### MOV

Syntax: `mov x -> r0, ..., rn`.

Action: Store the value of the register `x` on the registers `r0, ..., rn`.

#### High Level

```
mov r14 -> r15, r13
```

#### Microprogram

|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00011000|00000000000000000101|000|01110|11111|  00000000 |
|1 |111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

## HALT

_HALT_ is a special microinstruction to indicate that
the program has ended. In this design this opcode is achieved by seting all 64
bits of the microinstruction to 1.

This constant is avalible as a shortcut in the `CtrlStore` struct.
