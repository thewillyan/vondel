# Microinstruction

Microinstructions are stored in the control store and are fetched with the address
stode at MPC (Microprogram Counter).

A Vondel microinstruction has 64 bits the following format:

|   NEXT  |  JAM   |   ALU  |  C BUS  |  MEM   |   A    |   B    | IMMEDIATE |
|:-------:|:------:|:------:|:-------:|:------:|:------:|:------:|:---------:|
| 9 bits  | 3 bits | 8 bits | 20 bits | 3 bits | 5 bits | 5 bits |  8 bits   |


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

TODO!

## C BUS

TODO!

## MEM

TODO!

## A and B

`A` and `B` are fields that control which register writes to the A and B bus
respectively, but, since the A and B bus are connected directly to the A and B
inputs of the ALU, they can also be viewed as which register goes as a entry
to the ALU operation.

The values of `A` and `B` and which register they enable (NONE,
represents that none of them writes to the respective BUS, so the value of the
bus is 0) is shown below:

### Output to BUS A

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

### Output to BUS B

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

## Terminate

_TERMINATE_ is a special microinstruction to indicate that
the program has ended. In this design this opcode is achieved by seting all
bits of the microinstruction to 1.

This constant is avalible as a shortcut in the `CtrlStore` struct.

## Instruction Implementations

In this section we will show how some of the microinstruction of the Vondel
Language are implemented in the microprogram.

Operations summary:

- Arithmetic
  - [ADD](#add)
  - [SUB](#sub)
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
storing `0x0A` into MBR and then use `JMPC` to jump to that address`.
For that the number 10 (`0x0A`) must be stored at memory and its address
should be stored in PC. So, let's store 10 in the current address of the PC,
by default the PC is 0, so let's store it there.


|ID|   NEXT  |JAM|   ALU  |       C BUS        |MEM|  A  |  B  | IMMEDIATE |
|:-|:-------:|:-:|:------:|:------------------:|:-:|:---:|:---:|:---------:|
|0 |000000001|000|00000000|00000000000000000000|010|11111|11111|  00000000 |
|1 |000000010|100|00000000|00000000000000000000|000|11111|11111|  00000000 |
|- |    -    | - |    -   |         -          | - |  -  |  -  |  00000000 |
|10|111111111|111|11111111|11111111111111111111|111|11111|11111|  11111111 |

In a nutshell the operations are the following:

1. Fetch the memory.
2. Set `JMPC` to 1.

### TODO

- SLT
- NOP
- Beq
- Bne
- Blt
- Bge
- Mul
