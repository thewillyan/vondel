# Comparision with proposed Assembler and Language

## Output

The first and more obvious difference between our project and the proposed, is that we generate the `Microinstruction` itself
and not a tuple `(Opcode, Byte_addr)` that can be used by the microarchitecture.

We choose this method because of the poor performance that is accomplished using `fetch` operation for getting the next instruction

A common operation of `goto` or `jal` can be perfomed on a single instruction in Vondel because we know the next address without having to use `fetch`

| Index |                                 Vondel                                 | Proposed                               |
| :---: | :--------------------------------------------------------------------: | :------------------------------------- |
|   0   | 0b000000010_100_00000000_00000000000000000000_000_11111_11111_00000000 | 0b00001110_000_00110101_001000_001_001 |
|   1   |                                                                        | 0b00000000_100_00010100_001000_001_010 |

Another difference is the way that we interact with `RAM`, on Vondel we produce a `.ram` file that it's separated from the firmware file,
this, by itself, can reduce early gotos that are common in the proposed one. Because all words can be placed early and stop the program without executing anything

## Combinations

Vondel has more than `700000000` possibilities for creating instructions.
This is because of our 23 register that can be combined in any way on `C_Bus`, 24 options for `A_bus` and 19 for `B_bus`. Where every register of
**A** or **B** bus can be replaced with a immediate or an address to a label.

On the proposed assembler, we only have operations with register `X` and everything in centered around him, limiting the way a programmer can
interact with a microarchitecture.

## Extensibility

Adding new **Mnemonics**, **Registers**, **PseudoOps**, **Sections** or updating previous one is extremelly easy on Vondel.

Because of `Rust` powerful type-system, any new changes that you make to this project will be captured by both our unit tests and
Rust itself, leading to a more robust project.

The proposed one was written in python without any tests

## Fine Grained Syntax

On Vondel, we can define the syntax of our Language in any way that you want.
Because we created a parsing stage, our syntax will be defined there.

A syntax pattern that would be hard to do in the proposed one is our `write` function.

```asm
write 123 <- t0
write addr <- a1
```

Most Vondel instructions can receive up to **23** registers as destination, but this one only receives `label` or `immediate` for storing the result
and it must have an `register` as input

Implementing this on the proposed one would require to remake the entire bussiness logic that consists of `opcode, dest, source`

On Vondel we just create a arm on the `match statement` and it's done

## Error Handling and Diagnostics

One of the most important features of a interpreter/compiler/assembler is how well it handle errors and show this information for a user.

On the proposed one, if something goes wrong the parser still tries to generate a file and just prints that have happened an error like this:

```sh
Error of syntax on line 15
```

We improved this error system using Rust powerful type-system that cries until we handle it on a good manner using this approachs:

1. Show not only a line but also which token is required and at which column
2. If some operation is forbidden like writing to MBR we send an error message to the user

```sh
Expected "Assign" found "Comma"
Line 13, column 9

Cannot write to MAR on C Bus
Line 18, column 9
```

## Immediate Support

On Vondel we support Immediates, that are a type of data used in microarchitecture to provide immediate values or constants as operands in instructions.
They allow for the inclusion of constant values directly within the instruction itself, without the need to load them from memory or registers.

It's not implemented on the proposed one
