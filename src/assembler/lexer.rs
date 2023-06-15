use std::{iter::Peekable, str::Chars};

use crate::assembler::tokens::AsmToken;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    cur_line: usize,
    cur_column: usize,
    cur_char: char,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut l = Lexer {
            chars: input.chars().peekable(),
            cur_line: 1,
            cur_column: 0,
            cur_char: '\0',
        };
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        match self.chars.next() {
            Some(c) => {
                self.cur_char = c;
                match c {
                    '\n' => {
                        self.cur_line += 1;
                        self.cur_column = 1;
                    }
                    '\t' => self.cur_column += 4,
                    _ => self.cur_column += 1,
                };
            }
            None => self.cur_char = '\0',
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.cur_char.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    fn ignore_comment(&mut self) {
        if self.cur_char == '#' || self.cur_char == ';' {
            self.read_char();
            while self.cur_char != '\0' && self.cur_char != '\n' {
                self.read_char();
            }
            self.skip_whitespace();
        }
    }

    fn read_identifier(&mut self) -> AsmToken {
        let mut ident = String::from(self.cur_char);
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.read_char();
            } else {
                break;
            }
        }
        AsmToken::name_to_tok(ident)
    }

    fn tokenizer(&mut self) -> AsmToken {
        match self.cur_char {
            ':' => AsmToken::Colon,
            ',' => AsmToken::Comma,
            c if c.is_alphabetic() => self.read_identifier(),
            '\0' => AsmToken::Eof,
            _ => AsmToken::Illegal,
        }
    }

    pub fn next_token(&mut self) -> AsmToken {
        self.skip_whitespace();
        self.ignore_comment();
        let tok = self.tokenizer();
        self.read_char();
        tok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_whitespace() {
        let input = "    \t\n tubias";
        let mut l = Lexer::new(input);
        assert_eq!(l.next_token(), AsmToken::Identifier("tubias".to_string()));
    }

    #[test]
    fn ignore_comments() {
        let input = "# this is a comment\n tubias ; another comment here \n another_tubias #comment until end";
        let mut l = Lexer::new(input);
        assert_eq!(l.next_token(), AsmToken::Identifier("tubias".to_string()));
        assert_eq!(
            l.next_token(),
            AsmToken::Identifier("another_tubias".to_string())
        );
        assert_eq!(l.next_token(), AsmToken::Eof);
    }
}
