use super::lexer::*;
use std::io::{self, Write};

const PROMPT: &str = "🔥 ";

pub fn start() {
    loop {
        print!("{PROMPT}");
        io::stdout().flush().unwrap();
        let mut buf = String::new();

        match io::stdin().read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => panic!("{e}"),
        };

        let mut lexer = Lexer::new(buf);
        loop {
            let token = lexer.next_token();
            if token == Token::EOF {
                break;
            }
            println!("{:?}", token);
        }
    }
}
