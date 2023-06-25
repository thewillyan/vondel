# Parser Design Decisions

The Vondel parser is designed to parse the Vondel programming language, which is built on top of the Monkey language.
The parser follows the Pratt parsing approach to handle expressions and statements in the program.

## Pratt Parser Explanation

Pratt parsing, also known as TDOP, is a parsing technique that uses a Recursive Descent Approach to parse expressions based on their precedence and associativity.

The main idea behind Pratt parsing is to associate each token or operator with two parsing functions: a prefix parsing function and an infix parsing function.
The prefix parsing function is responsible for parsing expressions that start with the token, while the infix parsing function handles expressions that involve the token as an operator.

Let's illustrate how that bitch works with this simple expression `(-5 + 3) * 2`

1. **Tokenization**:
   The expression is tokenized as follows:
   `(`, `-`, `5`, `+`, `3`, `)`, `*`, `2`

2. **Precedence and Associativity**:
   Let's assign the following precedence levels:
   `*` (highest), `+` (lower), `-` (lower)

3. **Prefix Parsing**:
   The prefix parsing function for the `(` token creates a grouping expression.

4. **Prefix Parsing for `-`**:
   The parser encounters the `-` token and checks if it's a prefix operator. Since it is, the parser creates a unary expression for it.

5. **Infix Parsing**:
   The parser encounters the `)` token and skips it since it doesn't have an associated infix parsing function.

6. **Right Binding Power**:
   The right binding power for `*` is the same as its precedence level.

7. **Right-hand Side Expression**:
   The parser invokes the infix parsing function for `*` and creates a binary expression with the `*` operator and the right-hand side expression.

8. **Recursive Descent**:
   The parser examines the next token (`2`) and creates an integer expression for it.

9. **Parsing Complete**:
   The parsing process is complete, resulting in a fully parsed expression.

Here's the visualization of the Pratt parsing process for the expression `(-5 + 3) * 2`:

```
       *
      / \
     +   2
    / \
   -   3
    |
   5
```

## Error Handling

The parser uses the `anyhow` crate for error handling. It defines a custom `ParserError` enum to represent different parsing errors that can occur.
The `ErrorWithCtx` struct is used to associate the error message with the corresponding context in the source code.

## Precedence

The parser defines the Precedence enum to represent the precedence levels of different operators in the language.
The `precedence_of` function assigns a precedence level to each token type used in Pratt parsing.

## Statement Types

The parser defines the `StatementType` enum to represent different types of statements in the program.
This includes `Let statements` for variable declarations, `Return statements`, `Expression statements`, and `Block statements` for code blocks.

## Expression

The `Expression` module provides the definition and functionality for handling different types of expressions in the code.

### PrefixOp

The `PrefixOp` enum represents the prefix operators available in the language, including `Bang` and `Minus`.

### InfixOp

The `InfixOp` enum represents the infix operators available in the language, such as `Plus`, `Minus`, `Asterisk`, `Slash`, `Equal`, `NotEqual`, `LessThan`, and `GreaterThan`. It provides methods to retrieve the operator as a string representation.

### Expression

The `Expression` enum represents various types of expressions, including `Identifier`, `Integer`, `Prefix`, `Infix`, `Boolean`, `If`, `FunctionLiteral`, and `Call`. It also provides methods to get the type of the expression as a string and constructors for creating specific types of expressions.

#### Constructors

- `new_ident(ident: &String)`: Creates a new `Identifier` expression with the given identifier.
- `new_integer(int: &String) -> Result<Self>`: Creates a new `Integer` expression with the parsed integer value from the input string. Returns an error if the parsing fails.
- `new_prefix(op: &TokenType, exp: Expression) -> Result<Self>`: Creates a new `Prefix` expression with the specified operator and right-hand side expression. Returns an error if the operator is not allowed.
- `new_infix(left: Expression, op: &TokenType, right: Expression) -> Result<Self>`: Creates a new `Infix` expression with the specified left-hand side, operator, and right-hand side expressions. Returns an error if the operator is not allowed.
- `new_boolean(t: &TokenType) -> Result<Self>`: Creates a new `Boolean` expression with the specified boolean value. Returns an error if the boolean token is not allowed.
- `new_if(cond: Expression, cons: StatementType, alt: Option<StatementType>)`: Creates a new `If` expression with the condition, consequence, and optional alternative statements.
- `new_function(params: Vec<Expression>, block: StatementType)`: Creates a new `FunctionLiteral` expression with the specified parameters and block statement.
- `new_call(func: Expression, args: Vec<Expression>)`: Creates a new `Call` expression with the specified function expression and argument expressions.

## Program Structure

The parser uses the `Program` struct to store the parsed statements and errors. The `get_deez_program` method is responsible for parsing the entire program and populating the `Program` struct.

## Lexer Integration

The parser expects a slice of `TokenType` tokens as input, which are generated by a lexer. This allow us to do some kind of decoupling between lexing and parsing.

## Usage

To use the parser follow these steps:

1. Create a vector of tokens by hand or using a [lexer](/docs/src/interpreter/lexer.md).
   ```rust
   let input =  "let tubias = 123;"
   let toks = Lexer::new(input).get_deez_tokens();
   ```
2. Create a new `Parser` and pass this tokens as parameters.
   ```rust
   let mut parser = Parser::new(&toks);
   let program = parser.get_deez_program();
   ```
3. Do whatever you want with these statements or errors.

   ```rust
   // Access the parsed statements and handle errors if needed
   for statement in program.statements {
       // Process each statement...
   }

   // Handle parsing errors, if any
   for error in program.errors {
       // Handle each error...
   }
   ```

## Testing

The approach for testing this parser was decoupling it from the lexer. So that future internal changes of the current lexer can't interfere with our parse,
unless we change the contract between then that is `TokenType`

## Conclusion

In conclusion, the Vondel parser, based on the Pratt parsing approach, provides efficient and accurate parsing of expressions and statements in the Vondel programming language.
With its error handling capabilities, well-defined precedence levels, and support for various statement types,
the parser offers a solid foundation for the development and expansion of the Vondel language, ensuring reliable and effective parsing of code structures.
