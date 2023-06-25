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

## Assembly

Some details about the relation between the assembly code and the microarchitecture.

### Register nomeclature

In the assembly code the general purpose registers were renamed for convenience
in when writing code, the rename table is as following:

| Register | Assembly Name |
| --- | -- |
| R0  | Ra | 
| R1  | T0 |
| R2  | T1 |
| R3  | T2 |
| R4  | T3 |
| R5  | S0 |
| R6  | S1 |
| R7  | S2 |
| R8  | S3 |
| R9  | S4 |
| R10 | S5 |
| R11 | S6 |
| R12 | A0 |
| R13 | A1 |
| R14 | A2 |
| R15 | A3 |

**WARNING:** The registers `T0` - `T3` can be used in instructions like `mul` to
store temporary values, so the value of those can be changed in the instruction
implementations, therefore they are not guaranteed to have the value that you expect
using instruction like `lui` or `read`.

## Assembly Implementations

You can find some implementatinos examples of assembly code into the microarchitecture in the
[next chapter](./implementations.md).
