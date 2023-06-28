# Vondel

![Build Status](https://img.shields.io/github/actions/workflow/status/thewillyan/vondel/rust.yml)
![Issues](https://img.shields.io/github/issues/thewillyan/vondel)
![Pull requests](https://img.shields.io/github/issues-pr/thewillyan/vondel)
![License](https://img.shields.io/github/license/thewillyan/vondel)

A simple computer architecture, ISA, Assembler and Interpreter build with Rust.

## Usage

First off all, we need to build our binaries using:

```bash
cargo build -r --bins
```

Then for testing the demonstration programs we need to `assemble` it and run with our `uarch`.

We can accomplish this by just using the following commands:

### Running Factorial

```bash
cargo run -r --bin assembler -- -i programs/factorial_hardware.asm && \
cargo run -r --bin uarch -- --rom a.rom --ram a.ram
```

### Running Power

```bash
cargo run -r --bin assembler -- -i programs/power_hardware.asm && \
cargo run -r --bin uarch -- --rom a.rom --ram a.ram
```

### Running CSW

```bash
cargo run -r --bin assembler -- -i programs/csw.asm && \
cargo run -r --bin uarch -- --rom a.rom --ram a.ram
```

### Running Div

```bash
cargo run -r --bin assembler -- -i programs/div_hardware.asm && \
cargo run -r --bin uarch -- --rom a.rom --ram a.ram
```

## File Structure

The Vondel file structure follows a convetional Rust program, with a `src` folder that handles all of our source code.

More info on the chapter [File Structure](./files.md)

## Interpreter

The Vondel Interpreter was built on top of the [Monkey Language](https://monkeylang.org/) using the
book [Writing an Interpreter with GO](https://interpreterbook.com/) but on `Rust`.

More info on the chapter [Interpreter](./interpreter/README.md)

## Assembler

The `Vondel Assembler` was based on RISC V assemblers with our custom taste for Mnemonics and operations that our Uarch can accept

More info on the chapter [Assembler](./assembler/README.md)

## Microarchitecture

The basic idea behind Vondel's design is to follow some tips from the book _Structured
Computer Organization - Andrew S. Tanenbaum_ while implementing the ability to
perform operations even if the main clock is in a non-edge level in order to
reduce clock cycles.

More info on the chapter [Microarchitecture](./uarch/README.md)
