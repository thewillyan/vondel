# Rust Evaluator

The `RustEvaluator` is an implementation of the `Evaluator` trait specifically designed to evaluate Rust code. This document outlines the design decisions and considerations made in the implementation of the `RustEvaluator` struct.

## RustEvaluator Struct

The `RustEvaluator` struct represents an evaluator for Rust code. It provides methods for evaluating different types of expressions and statements in Rust.

## Usage

To use the `RustEvaluator` follow these steps:

1. Import the necessary modules and structs in your Rust file:

   ```rust
   use std::{cell::RefCell, rc::Rc};
   use anyhow::{Result};
   use crate::inter::{
       ast::expression::{InfixOp, PrefixOp},
       environment::Environment,
       object::Object,
       // Import other necessary modules as needed
   };
   use super::ast::*;
   // Import the RustEvaluator module
   use super::RustEvaluator;
   ```

2. Create an instance of the RustEvaluator:

   ```rust
   let evaluator = RustEvaluator::new();
   ```

3. Set up the environment and input for evaluation:

   ```rust
   let mut environment = Environment::new();
   let input = "1 + 2";
   ```

4. Pass the program to the eval() method of the RustEvaluator instance to evaluate it:
   ```rust
   let result = evaluator.eval(&program, &mut environment);
   ```
5. Handle the evaluation result using `Result`:
   ```rust
   match result {
       Ok(object) => {
           // Handle the evaluated object
           // ...
       }
       Err(error) => {
           // Handle the evaluation error
           // ...
       }
   }
   ```
6. Repeat steps 4 and 5 as needed to evaluate multiple programs or statements.

By following these steps, you can integrate the RustEvaluator into your Rust project and evaluate Rust code.

Feel free to modify the code snippets or the instructions to fit your specific use case.

### Methods

#### `new()`

The `new()` method creates a new instance of the `RustEvaluator`.

#### `map_boolean()`

The `map_boolean()` method maps a boolean value to the corresponding `Object` value used in the evaluator.

#### `eval_bang_operator()`

The `eval_bang_operator()` method evaluates the logical NOT (`!`) operator on the right operand.

#### `eval_minus_operator()`

The `eval_minus_operator()` method evaluates the arithmetic minus (`-`) operator on the right operand.

#### `eval_prefix_expression()`

The `eval_prefix_expression()` method evaluates a prefix expression (e.g., `-`, `!`) by applying the corresponding operator on the right operand.

#### `eval_infix_expression()`

The `eval_infix_expression()` method evaluates an infix expression (e.g., `+`, `-`, `*`, `/`, `<`, `>`, `==`, `!=`) by performing the operation on the left and right operands.

#### `is_truthy()`

The `is_truthy()` method checks if an object is truthy, meaning it evaluates to true.

#### `eval_if_expression()`

The `eval_if_expression()` method evaluates an `if` expression by evaluating the condition and executing the appropriate branch based on the result.

#### `eval_let_statement()`

The `eval_let_statement()` method evaluates a `let` statement by evaluating the value expression and storing it in the environment with the specified name.

#### `eval_expressions()`

The `eval_expressions()` method evaluates a list of expressions within the given environment.

#### `eval_fn_call()`

The `eval_fn_call()` method evaluates a function call expression by evaluating the function expression and the argument expressions, and executing the function with the provided arguments.

#### `eval_expression()`

The `eval_expression()` method evaluates an expression by dispatching to the appropriate evaluation method based on the expression type.

#### `eval_block_statements()`

The `eval_block_statements()` method evaluates a block of statements by iterating over each statement and evaluating them in order.

#### `eval_statements()`

The `eval_statements()` method evaluates a statement by dispatching to the appropriate evaluation method based on the statement type.

### Implementation of Evaluator Trait

The `RustEvaluator` struct implements the `Evaluator` trait, which defines the behavior of an evaluator. The `eval()` method is implemented to evaluate a program by iterating over each statement and evaluating them in order.

## Conclusion

The `RustEvaluator` provides a comprehensive implementation of an evaluator for Rust code. It handles different types of expressions and statements and provides accurate evaluation results.The design decisions made in the implementation ensure flexibility and extensibility for future enhancements or modifications.
