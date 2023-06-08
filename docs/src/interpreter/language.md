# Vondel Language Specification

This specification outlines the syntax and behavior of the Vondel language,
a simple programming language that supports arithmetic operations, functions, and lambda functions.

## Basics

- Vondel is a dynamically-typed language, meaning that variable types are inferred at runtime.
- All statements in Vondel end with a semicolon (`;`).
- Vondel is whitespace insensitive, meaning that spaces and line breaks are ignored except where necessary for separating tokens.

## Data Types

Vondel Suported Data Types:

- **Integers**: Represented by whole numbers without fractional or decimal parts.
- **Booleans**: Represented by the keywords `true` and `false`.
- **Null**: Represented by the keyword `null`.

## Variables

Variables in Vondel are dynamically typed and declared using the let keyword.

```vondel
let x = 10;
let name = "Alice";
let flag = true;
```

## Arithmetic Operations

Vondel supports the following arithmetic operations:

- Addition: `+`
- Subtraction: `-`
- Multiplication: `*`
- Division: `/`
- Modulo: `%`

```vondel
  let x = 10 + 5;
  let y = x * 2 - 3;
  let z = (x + y) / 2;
```

## Functions

Vondel allows the definition and invocation of functions.

### Function Definition

Functions are defined using the fn keyword followed by the parameter list in parentheses and the function body in curly braces.

```vondel
let add = fn(x, y) {
  return x + y;
};
```

### Function Invocation

To invoke a function, use the function name followed by arguments in parentheses.

```vondel
let result = add(3, 4); // result = 7
```

### Return Statement

The return statement is used to exit a function and optionally return a value.

```vondel
let multiply = fn(x, y) {
  return x * y;
};
```

## Lambda Functions

Vondel supports lambda functions, also known as anonymous functions or function literals.

### Lambda Function Definition

Lambda functions are defined using the fn keyword followed by the parameter list in parentheses and the function body in curly braces. They can be assigned to variables.

```vondel
let multiply = fn(x, y) {
  return x * y;
};

let square = fn(x) {
  return multiply(x, x);
};
```

### Lambda Function Invocation

To invoke a lambda function, use the function name followed by arguments in parentheses.

```vondel
let result = square(5); // result = 25
```

### Higher-Order Functions

Vondel supports higher-order functions, which are functions that can accept other functions as arguments or return functions as results.

```vondel
let applyFunc = fn(func, x, y) {
  return func(x, y);
};

let result = applyFunc(multiply, 3, 4); // result = 12
```

## Examples

```vondel
let add = fn(x, y) {
  return x + y;
};

let result = add(3, 4); // result = 7

let multiply = fn(x, y) {
  return x * y;
};

let square = fn(x) {
  return multiply(x, x);
};

let area = square(5); // area = 25
let fibonacci = fn(x) {
  if (x == 0) {
    0
  } else {
    if (x == 1) {
      1
    } else {
      fibonacci(x - 1) + fibonacci(x - 2);
    }
  }
};

fibonacci(9) // 34
```

## Limitations

The Vondel language described in this specification is a simplified version of a programming language and has several limitations, including but not limited to:

- Limited data types and operations.
- Lack of control flow statements such as loops and conditionals.
- Limited built-in functions and standard library.
- The language is intentionally designed to be minimalistic and educational, focusing on core concepts and syntax.

## References

The Monkey programming language: [monkey lang site](https://monkeylang.org/)
