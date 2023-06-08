# Interpreter

The Vondel Interpreter was built on top of the [Monkey Language](https://monkeylang.org/) using the
book [Writing an Interpreter with GO](https://interpreterbook.com/) but on `Rust` 😂

Monkey is a simple programming language designed for educational purposes, focusing on simplicity and ease of understanding.
This interpreter allows you to run Monkey code and see the results in real-time.

## Language Specs

The Vondel Language is simpler than a common Monkey Lang implementation because we need to implement a custom evaluator
using our own microarchitecture. And doing Strings, Structs and Vectors on this UArch will not be a good experience for us.

Basically, what we implemented in the Vondel Language, which includes integers, booleans, functions, and lambda functions,
you have a foundation for building a variety of programs.

More info on the sup-chapter [Language Specifications](./language.md).

## Lexer

Our lexer don't allow UTF-8 enconding for processing text and just rely on common ASCII.

It's a simple lexer that use a lot of String allocations instead of string slices for performance optimizations.
It also uses one of `Rust` most powerfull features `enums` for processing **non Identifier** tokens so that we can
rely on `Rust` strong type system.

More info on the sup-chapter [Lexer](./lexer.md)

## Parser

The parser for the Vondel language utilizes a powerful parsing technique called Pratt Parsing.
It allows for efficient and flexible parsing of expressions with varying levels of precedence so that we can parse complex expressions without too much hassle.

More info on the sup-chapter [Parser](./parser.md)

## Object

Because Vondel is a dynamic language we use `Objects` for an intermediate representation and evaluations

More info on the sup-chapter [Object](./object.md)

## Environment

It allows for the storage and retrieval of variables and their corresponding values in a flexible and efficient manner.
The module supports nested environments, enabling scoping and variable lookup in outer environments.

More info on the sup-chapter [Environment](./environment.md)

## Evaluator

For the evaluation module we just define a `EvaluationError` enum for handling most evaluation and runtime errors.
A `Evaluator` trait for creating custom evaluators and `evaluate_buffer` **fn** that handles receives anything that implements `Evaluator`, a `Program` and return a prints the result of this evaluation

We also define two differente evaluators for our users:

1. [Rust Evaluator](./evaluator/rust.md) - that uses Rust for doing all evaluations
2. [Custom Evaluator](./evaluator/custom.md) - that uses our custom microarchitecture for evaluting these ASTs

More info on the sup-chapter [Evaluator](./evaluator/README.md)
