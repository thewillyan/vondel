# Assembler

The `Vondel Assembler` was based on RISC V assemblers with our custom taste for Mnemonics and operations that our Uarch can accept

This assembler produce 2 files a `.ram` and a `.rom` that can be used for our microarchitecture needs where:

- **.ram**: It's the ram dump produced by `.data` sections
- **.rom**: It's the firmware made by `.text` sections

## Usage

To use the `Vondel Assembler` you can open your terminal and type

```sh
cargo run -r assembler --  -i input.asm -o output
```

This commmand will generate lex, parse and evaluate the `input.asm` and therefore produce `output.ram` and `output.rom` for further usage

> If the output argument isn't provided the default output file will begin with `a`

## Language Specifications

Our language specs are similar to RISC V, but with some tweaks

If you came from there you'll be in home

```asm
.data
    tubias: .word 420
    test: .byte 69
    res: .word 42069

.text
main:
    addi s0 <- test
    lui s1 <- 254
    add a0,a1,a2,a3 <- t1, t2
    beq t1, a0, done


done:
    read s2 <- tubias
    write res <- t1
    halt
```

More info on the sup-chapter [Language Specs](./specs.md)

## Lexer

Just a common lexer that we'll ignore comments and parse all necessary tokens for further steps

More info on the sup-chapter [Lexer](./lexer.md)

## Parser

Here we define our syntax that is described on [Language Specs](./specs.md), if any step here fail we should the user
the error and context

More info on the sup-chapter [Parser](./parser.md)

## Evaluator

We generate our `ROM` and `RAM` for our microarchitecture, it handles the logic of our firmware and all variables
that are beeing used

More info on the sup-chapter [Evaluator](./evaluator.md)

## Comparision

The comparision between our language and assembler with the ones that our teacher proposed at class

The `key` difference between then are:

1. Our assembler is easier to extend and define a fine grained syntax than the proposed one.
2. We have a error handling with more context for the user
3. We build the microinstruction itself while our teacher use opcode to navegate on the firmware own microinstructions
4. We support immediates that can speed the process a lot

More info on the sup-chapter [Comparision](./comparision.md)
