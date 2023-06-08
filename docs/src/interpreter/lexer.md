# Lexer Design Decision

The lexer provided is responsible for tokenizing input strings into a sequence of tokens. This README explains the design decisions made in implementing the lexer.

## Overall Design

The lexer is implemented as a struct called `Lexer` with the following fields:

- `ch`: Represents the current character being processed.
- `input`: Represents the input string as a vector of bytes.
- `read_position`: Tracks the current position while reading the input.
- `position`: Tracks the position of the current character being processed.

The `Lexer` struct also includes several methods to perform different tasks:

- `new`: Initializes a new `Lexer` instance with the given input string.
- `skip_whitespace`: Skips whitespace characters until a non-whitespace character is encountered.
- `peek_char`: Returns the next character in the input without consuming it.
- `read_char`: Reads the next character from the input and updates the lexer's internal state.
- `next_token`: Returns the next token from the input.
- `parse_operator`: Parses an operator token based on the current and next characters in the input.
- `tokenizer`: Tokenizes the current character based on its type.
- `read_name`: Reads an identifier token from the input.
- `read_number`: Reads an integer number token from the input.
- `get_deez_toks`: Returns a vector of all tokens found in the input.

The design follows a simple and straightforward approach to tokenizing the input string.

## Tokenization

The `tokenizer` method is responsible for tokenizing the current character based on its type. It uses pattern matching to identify different types of characters and returns the corresponding token type.

- Whitespace characters are skipped using the `skip_whitespace` method.
- Single-character tokens like commas, semicolons, parentheses, and squirlies are identified directly.
- Operator tokens are parsed by the `parse_operator` method, which takes into account both single and double-character operators.
- Identifiers are recognized by checking if the current character is alphabetical or an underscore. The `read_name` method is called to read the complete identifier.
- Integer numbers are recognized by checking if the current character is a digit. The `read_number` method is called to read the complete number.
- Any other character is considered illegal and is represented by an `Illegal` token type.

## Usage

To use the lexer, follow these steps:

1.  Import the `Lexer` struct and the `TokenType` enum from the appropriate module or file.

2.  Create a new instance of the `Lexer` by calling the `new` method and passing the input string as a parameter.

    ```rust
    let input = String::from("let x = 5 + 3;");
    let mut lexer = Lexer::new(input);
    ```

3.  Retrieve tokens from the lexer by calling the next_token method. It returns the next token in the input string.

    ```rust
    let token = lexer.next_token();

    ```

4.  Continue calling next_token to retrieve subsequent tokens until you reach the end of the input. The end of input is indicated by the TokenType::Eof token.
    ```rust
    loop {
        let token = lexer.next_token();
        if token == TokenType::Eof {
            break;
            }
        // Process the token as needed
    }
    ```
5.  Or you can use `get_deez_toks` method to return all tokens at once as a `Vec<TokenType>`

    ```rust
    let all_toks = lexer.get_deez_toks();
    ```

6.  Use the retrieved tokens for further processing or analysis based on your specific requirements.

Please note that this example assumes you have the appropriate code structure and dependencies in place for the lexer to work correctly. Adjust the code snippets based on your specific implementation.

## Testing

The lexer includes a set of unit tests implemented using the `#[cfg(test)]` attribute and the `mod tests` block.
These tests ensure the correctness of the lexer implementation by verifying the tokenization of different input strings.

The tests cover various scenarios, including skipping whitespace, reading identifiers and numbers,
handling operators, and tokenizing complex input strings with multiple tokens.

## Conclusion

The lexer design follows a modular and intuitive approach to tokenize input strings.
It provides flexibility to extend the lexer's functionality if required.
The unit tests provide confidence in the correctness of the lexer implementation.
