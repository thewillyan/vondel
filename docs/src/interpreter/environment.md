# Environment

The `Environment` module provides functionality for storing variables and their corresponding values.

## Features

- **Variable Storage**: The `Environment` struct uses a `HashMap` to store variables and their values.
- **Nested Environments**: The module supports nested environments, allowing for scoping and variable lookup in outer environments.
- **Argument Matching**: The `set_arguments_to_env` function sets arguments to the environment, matching them with corresponding parameters.
- **Error Handling**: The module uses the `anyhow` crate for error handling, providing detailed error messages for various scenarios.

## Usage

To create a new environment, use the `Environment::new()` function. To create an environment with an outer environment, use `Environment::new_with_outer(outer)`.

To set arguments to the environment, use the `set_arguments_to_env(args, params)` function, providing a vector of arguments and parameters.

To retrieve the value of a variable, use the `get(name)` function, providing the variable name. It automatically looks up the variable in outer environments if it's not found in the current environment.

To set the value of a variable, use the `set(name, value)` function, providing the variable name and value.

## Examples

```rust
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use anyhow::{bail, Result};

use crate::inter::{ast::Expression, evaluator::EvaluationError};

use super::object::Object;

/// Represents an environment that stores variables and their corresponding values.
#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    // ... Implementation details ...
}

// Example usage
fn main() {
    let mut environment = Environment::new();

    environment.set(
        &Expression::Identifier("x".to_string()),
        Object::Integer(42),
    ).unwrap();

    let result = environment.get("x").unwrap();
    println!("Value of 'x': {}", result);
}
```
