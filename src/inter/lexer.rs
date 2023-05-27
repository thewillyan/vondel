pub type TokenType = Token;
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Integer(String),
    ILLEGAL,
    EOF,
    //Punctuation
    Comma,
    Semicolon,
    LParen,
    RParen,
    LSquirly,
    RSquirly,
    //Keywords
    True,
    False,
    Return,
    If,
    Else,
    Function,
    Let,
    //Operators
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    LessThan,
    GreaterThan,
    LessEQ,
    GreaterEQ,
    Equal,
    NotEqual,
}

pub struct Lexer {
    ch: u8,
    input: Vec<u8>,
    read_position: usize,
    position: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut l = Lexer {
            ch: b' ',
            input: input.into_bytes(),
            read_position: 0,
            position: 0,
        };
        l.read_char();
        l
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    fn peek_char(&self) -> char {
        let mut c = '\0';
        if self.read_position < self.input.len() {
            c = self.input[self.read_position] as char;
        };
        c
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = b'\0';
        } else {
            self.ch = self.input[self.read_position];
            self.position = self.read_position;
            self.read_position += 1;
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let tok = self.tokenizer();
        self.read_char();
        tok
    }

    fn parse_operator(&mut self) -> Token {
        let curr = self.ch;
        let doubled = self.peek_char() == '=';
        if doubled {
            self.read_char();
        }
        match (curr, doubled) {
            (b'=', true) => Token::Equal,
            (b'=', false) => Token::Assign,
            (b'<', true) => Token::LessEQ,
            (b'<', false) => Token::LessThan,
            (b'>', true) => Token::GreaterEQ,
            (b'>', false) => Token::GreaterThan,
            (b'!', true) => Token::NotEqual,
            (b'!', false) => Token::Bang,
            (b'+', false) => Token::Plus,
            (b'-', false) => Token::Minus,
            (b'*', false) => Token::Asterisk,
            (b'/', false) => Token::Slash,
            _ => Token::ILLEGAL,
        }
    }

    fn tokenizer(&mut self) -> Token {
        match self.ch {
            b'\0' => Token::EOF,
            b',' => Token::Comma,
            b';' => Token::Semicolon,
            b'(' => Token::LParen,
            b')' => Token::RParen,
            b'{' => Token::LSquirly,
            b'}' => Token::RSquirly,
            b'!' | b'=' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' => self.parse_operator(),
            c if c.is_ascii_alphabetic() || c == b'_' => self.read_name(),
            c if c.is_ascii_digit() => self.read_number(),
            _ => Token::ILLEGAL,
        }
    }

    fn read_name(&mut self) -> Token {
        let start = self.position;
        while self.read_position != self.input.len()
            && self.input[self.read_position].is_ascii_alphabetic()
        {
            self.read_char();
        }
        let ident = String::from_utf8(self.input[start..self.read_position].to_vec()).unwrap();
        match ident.as_str() {
            "true" => Token::True,
            "false" => Token::False,
            "fn" => Token::Function,
            "let" => Token::Let,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            _ => Token::Ident(ident),
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        while self.read_position != self.input.len()
            && self.input[self.read_position].is_ascii_digit()
        {
            self.read_char();
        }
        Token::Integer(String::from_utf8(self.input[start..self.read_position].to_vec()).unwrap())
    }

    pub fn get_all_toks(&mut self) -> Vec<TokenType> {
        let mut toks = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == Token::EOF {
                toks.push(tok);
                break;
            }
            toks.push(tok);
        }
        toks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_whitespace() {
        let mut lexer = Lexer::new(String::from("               {"));
        assert_eq!(lexer.next_token(), Token::LSquirly);
    }

    #[test]
    fn read_name() {
        let mut lexer = Lexer::new(String::from("tubiaasdasdasd"));
        assert_eq!(
            lexer.next_token(),
            Token::Ident(String::from("tubiaasdasdasd"))
        );
    }

    #[test]
    fn read_number() {
        let mut lexer = Lexer::new(String::from("123123123"));
        assert_eq!(
            lexer.next_token(),
            Token::Integer(String::from("123123123"))
        );
    }

    #[test]
    fn no_whitespace_idents() {
        let mut l = Lexer::new(String::from("five=5;"));
        let v = vec![
            Token::Ident(String::from("five")),
            Token::Assign,
            Token::Integer(String::from("5")),
            Token::Semicolon,
            Token::EOF,
        ];

        for i in v.into_iter() {
            assert_eq!(i, l.next_token());
        }
    }

    #[test]
    fn operators() {
        let mut l = Lexer::new(String::from(
            r#"10 == 10;
            10 != 9;
            10 >= 9;
            9 <= 10;"#,
        ));
        let v = vec![
            Token::Integer(String::from("10")),
            Token::Equal,
            Token::Integer(String::from("10")),
            Token::Semicolon,
            Token::Integer(String::from("10")),
            Token::NotEqual,
            Token::Integer(String::from("9")),
            Token::Semicolon,
            Token::Integer(String::from("10")),
            Token::GreaterEQ,
            Token::Integer(String::from("9")),
            Token::Semicolon,
            Token::Integer(String::from("9")),
            Token::LessEQ,
            Token::Integer(String::from("10")),
            Token::Semicolon,
            Token::EOF,
        ];

        for i in v.into_iter() {
            assert_eq!(i, l.next_token());
        }
    }

    #[test]
    fn next_token() {
        let input = String::from(
            r#"let five = 5;
            let add = fn(x, y) {
            x + y;
            };
            if (5 < 10) {
                return true;
            } else {
                return false;
            }
            "#,
        );
        let v = vec![
            Token::Let,
            Token::Ident(String::from("five")),
            Token::Assign,
            Token::Integer(String::from("5")),
            Token::Semicolon,
            Token::Let,
            Token::Ident(String::from("add")),
            Token::Assign,
            Token::Function,
            Token::LParen,
            Token::Ident(String::from("x")),
            Token::Comma,
            Token::Ident(String::from("y")),
            Token::RParen,
            Token::LSquirly,
            Token::Ident(String::from("x")),
            Token::Plus,
            Token::Ident(String::from("y")),
            Token::Semicolon,
            Token::RSquirly,
            Token::Semicolon,
            Token::If,
            Token::LParen,
            Token::Integer(String::from("5")),
            Token::LessThan,
            Token::Integer(String::from("10")),
            Token::RParen,
            Token::LSquirly,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::RSquirly,
            Token::Else,
            Token::LSquirly,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::RSquirly,
            Token::EOF,
        ];
        let mut l = Lexer::new(input);

        for i in v.into_iter() {
            assert_eq!(i, l.next_token());
        }
    }
}
