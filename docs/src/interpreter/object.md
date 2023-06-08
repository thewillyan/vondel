# Object

The `Object` module in the Vondel language is designed to handle various types of objects used in the language, including integers, booleans, return values, null values, and functions. It provides a `Object` enum with associated data for each object type. The module defines methods to inspect objects and retrieve their string representation. The `Object` enum implements the `Debug`, `PartialEq`, and `Clone` traits for debugging, equality comparison, and cloning, respectively. Additionally, it provides a `type_as_string()` method to retrieve the type of the object as a string for error handling.

## Object Enum

The `Object` enum represents the different types of objects in the Vondel language. It has the following variants:

- `Integer(i64)`: Represents an integer value.
- `Boolean(bool)`: Represents a boolean value.
- `ReturnValue(Box<Object>)`: Represents a return value from a function.
- `Null`: Represents a null value.
- `Function { params: Vec<Expression>, body: StatementType, env: Rc<RefCell<Environment>> }`: Represents a function object, storing the function's parameters, body, and environment.

## Object Methods

The `Object` module provides the following methods:

- `inspect() -> String`: Returns a string representation of the object.
- `type_as_string() -> &'static str`: Returns the type of the object as a string for error handling.

## Display Formatting

The `Object` enum implements the `fmt::Display` trait to provide a formatted string representation for display purposes. The `Display` implementation formats the object based on its variant, converting it to a string representation.

```rust
impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the object based on its variant
        // ...
    }
}

```

