# Evaluator

The `evaluator` module provides the definition and functionality for evaluating code using different evaluators.
This document outlines the design decisions and considerations made in the implementation of the evaluator module.

## EvaluationError Enum

The `EvaluationError` enum represents possible errors that can occur during evaluation.
It includes various error variants such as `MissingIntegerToInvert`, `MismatchedTypesInfix`, `UnallowedBooleanComparisonOperator`, `IdentifierNotFound`, `ExpectedIdentifier`, and `WrongNumberOfArguments`.
These errors cover different scenarios where evaluation can fail and provide descriptive error messages.

## Evaluator Trait

The `Evaluator` trait defines the behavior of an evaluator. It includes a single method, `eval`, which takes an AST node and an environment and returns the resulting `Object` or an error if evaluation fails.
Implementations of this trait provide the specific evaluation logic for different languages or custom evaluators.

## Evaluator Implementations

The evaluator module includes two sub-modules, [custom](./custom.md) and [rust](./rust.md) , which contain different implementations of the `Evaluator` trait.
These implementations can be used to evaluate code written in different languages or using custom evaluation logic.

## evaluate_buffer Function

The `evaluate_buffer` function is a utility function that simplifies the evaluation process. It takes a boxed trait object implementing the `Evaluator` trait and an input string to evaluate.
The function performs the lexing, parsing, and evaluation steps, and prints the resulting object or error message. It can be used as a convenient way to evaluate code using different evaluators.

## Usage

To use our evalutor follow theses steps:

1.  Create a `Custom Custom` evaluator or just use our [custom](./custom.md) and [rust](./rust.md) builtin evaluators
    ```rust
    use my_evaluator::{Evaluator, evaluate_buffer};
    // Define a custom evaluator
    struct MyEvaluator;
    impl Evaluator for MyEvaluator {
        // Implement the `eval` method
        // ...
    }
    ```
2.  Create a new Evaluator instance
    ```rust
    let my_eval = Box::new(MyEvaluator::new());
    let rust_eval = Box::new(RustEvaluator::new());
    let custom_eval = Box::new(CustomEvaluator::new();
    ```
3.  Use `evaluater_buffer` fn or create your own logic for parsing `Program`
    ```rust
    let input  = String::from("777 * 777");
    println!("my_eval res: {}",  evaluate_buffer(my_eval, input)?;
    println!("rust_eval res: {}",  evaluate_buffer(rust_eval, input)?;
    println!("custom_eval res: {}",  evaluate_buffer(custom_eval, input)?;
    ```

## Testing

The `Evaluator` module provides a set of tests to ensure the correctness of the evaluators and their functionalities.
The tests cover various scenarios, including error handling, function calls, and decoupling from the parser and lexer.

## Conclusion

In conclusion, the Evaluator module plays a crucial role in interpreting and evaluating Rust code.
It provides an essential component for executing programs, performing expressions, handling errors, and enabling function calls.
By decoupling from the parser and lexer, the evaluators ensure flexibility and compatibility with various parsing and lexing implementations.
The comprehensive test suite included in the module verifies the correctness of the evaluators, covering error handling, function calls, and specific behaviors of the RustEvaluator and CustomEvaluator implementations.
With the Evaluator module, developers can confidently evaluate Rust code and rely on its robustness and accuracy in producing the expected results.
