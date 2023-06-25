# Lexer

This lexer is specifically designed for a custom microarchitecture and includes RISC-V opcodes. It provides tokenization functionality for assembly code.

## Usage

To utilize the Lexer in your project, follow these steps:

1. Include the lexer code in your project's source files.
2. Import the necessary modules and dependencies, such as `std::{iter::Peekable, rc::Rc, str::Chars}`.
3. Instantiate a `Lexer` object, passing the input source code as a string to the new function.
4. Use the provided methods to tokenize the source code
   - `next_token`: Retrieves the next token in the source code.
   - `next_with_ctx`: Retrieves the next token with context information, including line and column numbers.
   - `get_deez_toks`: Retrieves all tokens in the source code as a vector.
   - `get_deez_toks_w_ctx`: Retrieves all tokens with context information as a vector.

## Example

```rust
use crate::assembler::tokens::{AsmToken, TokWithCtx};

let input_code = "...";  // Provide your assembly code here
let mut lexer = Lexer::new(&input_code);

loop {
    let token = lexer.next_token();
    if token == AsmToken::Eof {
        break;
    }
    println!("Token: {:?}", token);
}
```
