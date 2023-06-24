`AsmEvaluator` is a custom microarchitecture evaluator implemented in Rust.
It provides functionality for evaluating assembly code written in a custom instruction set architecture.

# Usage

To use the `AsmEvaluator`, follow these steps:

1. Create a new instance of `AsmEvaluator` using the new method:

```rust
let mut evaluator = AsmEvaluator::new();
```

2. Call the `evaluate_buffer` method to evaluate a buffer containing assembly code:

```rust
let buffer = r"
.text
main:
    lui t1 <- 77
";
let result = evaluator.evaluate_buffer(buffer);
```

The evaluate_buffer method returns a `Result` containing the control store (`CtrlStore`) and a reference to the evaluated memory `(&[u32])`. You can handle the result accordingly.

3. Alternatively, you can evaluate a Program directly by calling the eval_program method:

```rust
let program = get_program(); // Obtain the program somehow
let result = evaluator.eval_program(program);
```

The eval_program method follows a similar pattern as `evaluate_buffer` but takes a `Program` as input.

4. You can also evaluate individual sections of the program using the `eval` method:

```rust
let sections = get_sections(); // Obtain the sections somehow
let control_store = evaluator.eval(&sections);
```

The eval method takes a reference to the `Sections` enum, which can contain either a `TextSection` or a `DataSection`. It returns the `CtrlStore` generated from the evaluation.

# Structure

The `AsmEvaluator` struct has the following fields:

- `values`: A HashMap mapping labels to values (u8).
- `addr`: A HashMap mapping labels to addresses (u8).
- `ram`: A vector representing the random access memory.
- `unreachable`: A vector containing tuples of unreachable labels, their corresponding control store addresses, and microinstructions.
  The struct implements the Default trait using the derive attribute.

The `AsmEvaluator` struct provides the following methods:

- `new`: Creates a new instance of AsmEvaluator.
- `evaluate_buffer`: Evaluates a buffer of assembly code and returns the control store and evaluated memory.
- `eval_program`: Evaluates a Program and returns the control store and evaluated memory.
- `eval`: Evaluates individual sections of the program and returns the control store.
  Additionally, there are internal helper methods for evaluating data and text segments, resolving unreachable instructions, and evaluating different types of instructions.

# Error Handling

If there are errors while parsing the program, the `evaluate_buffer` method will print the errors and return a `Result` with an error message. It's important to handle this case to ensure proper error reporting and handling.
