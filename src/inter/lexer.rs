use super::tokens::TokenType;

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

    pub fn next_token(&mut self) -> TokenType {
        self.skip_whitespace();
        let tok = self.tokenizer();
        self.read_char();
        tok
    }

    fn parse_operator(&mut self) -> TokenType {
        let curr = self.ch;
        let doubled = self.peek_char() == '=';
        if doubled {
            self.read_char();
        }
        match (curr, doubled) {
            (b'=', true) => TokenType::Equal,
            (b'=', false) => TokenType::Assign,
            (b'<', true) => TokenType::LessEQ,
            (b'<', false) => TokenType::LessThan,
            (b'>', true) => TokenType::GreaterEQ,
            (b'>', false) => TokenType::GreaterThan,
            (b'!', true) => TokenType::NotEqual,
            (b'!', false) => TokenType::Bang,
            (b'+', false) => TokenType::Plus,
            (b'-', false) => TokenType::Minus,
            (b'*', false) => TokenType::Asterisk,
            (b'/', false) => TokenType::Slash,
            _ => TokenType::Illegal,
        }
    }

    fn tokenizer(&mut self) -> TokenType {
        match self.ch {
            b'\0' => TokenType::Eof,
            b',' => TokenType::Comma,
            b';' => TokenType::Semicolon,
            b'(' => TokenType::LParen,
            b')' => TokenType::RParen,
            b'{' => TokenType::LSquirly,
            b'}' => TokenType::RSquirly,
            b'!' | b'=' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' => self.parse_operator(),
            c if c.is_ascii_alphabetic() || c == b'_' => self.read_name(),
            c if c.is_ascii_digit() => self.read_number(),
            _ => TokenType::Illegal,
        }
    }

    fn read_name(&mut self) -> TokenType {
        let start = self.position;
        while self.read_position != self.input.len()
            && self.input[self.read_position].is_ascii_alphabetic()
        {
            self.read_char();
        }
        let ident = String::from_utf8(self.input[start..self.read_position].to_vec()).unwrap();
        match ident.as_str() {
            "true" => TokenType::True,
            "false" => TokenType::False,
            "fn" => TokenType::Function,
            "let" => TokenType::Let,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            _ => TokenType::Ident(ident),
        }
    }

    fn read_number(&mut self) -> TokenType {
        let start = self.position;
        while self.read_position != self.input.len()
            && self.input[self.read_position].is_ascii_digit()
        {
            self.read_char();
        }
        TokenType::Integer(
            String::from_utf8(self.input[start..self.read_position].to_vec()).unwrap(),
        )
    }

    pub fn get_deez_toks(&mut self) -> Vec<TokenType> {
        let mut toks = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == TokenType::Eof {
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
        assert_eq!(lexer.next_token(), TokenType::LSquirly);
    }

    #[test]
    fn read_name() {
        let mut lexer = Lexer::new(String::from("tubiaasdasdasd"));
        assert_eq!(
            lexer.next_token(),
            TokenType::Ident(String::from("tubiaasdasdasd"))
        );
    }

    #[test]
    fn read_number() {
        let mut lexer = Lexer::new(String::from("123123123"));
        assert_eq!(
            lexer.next_token(),
            TokenType::Integer(String::from("123123123"))
        );
    }

    #[test]
    fn no_whitespace_idents() {
        let mut l = Lexer::new(String::from("five=5;"));
        let v = vec![
            TokenType::Ident(String::from("five")),
            TokenType::Assign,
            TokenType::Integer(String::from("5")),
            TokenType::Semicolon,
            TokenType::Eof,
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
            TokenType::Integer(String::from("10")),
            TokenType::Equal,
            TokenType::Integer(String::from("10")),
            TokenType::Semicolon,
            TokenType::Integer(String::from("10")),
            TokenType::NotEqual,
            TokenType::Integer(String::from("9")),
            TokenType::Semicolon,
            TokenType::Integer(String::from("10")),
            TokenType::GreaterEQ,
            TokenType::Integer(String::from("9")),
            TokenType::Semicolon,
            TokenType::Integer(String::from("9")),
            TokenType::LessEQ,
            TokenType::Integer(String::from("10")),
            TokenType::Semicolon,
            TokenType::Eof,
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
            TokenType::Let,
            TokenType::Ident(String::from("five")),
            TokenType::Assign,
            TokenType::Integer(String::from("5")),
            TokenType::Semicolon,
            TokenType::Let,
            TokenType::Ident(String::from("add")),
            TokenType::Assign,
            TokenType::Function,
            TokenType::LParen,
            TokenType::Ident(String::from("x")),
            TokenType::Comma,
            TokenType::Ident(String::from("y")),
            TokenType::RParen,
            TokenType::LSquirly,
            TokenType::Ident(String::from("x")),
            TokenType::Plus,
            TokenType::Ident(String::from("y")),
            TokenType::Semicolon,
            TokenType::RSquirly,
            TokenType::Semicolon,
            TokenType::If,
            TokenType::LParen,
            TokenType::Integer(String::from("5")),
            TokenType::LessThan,
            TokenType::Integer(String::from("10")),
            TokenType::RParen,
            TokenType::LSquirly,
            TokenType::Return,
            TokenType::True,
            TokenType::Semicolon,
            TokenType::RSquirly,
            TokenType::Else,
            TokenType::LSquirly,
            TokenType::Return,
            TokenType::False,
            TokenType::Semicolon,
            TokenType::RSquirly,
            TokenType::Eof,
        ];
        let mut l = Lexer::new(input);

        for i in v.into_iter() {
            assert_eq!(i, l.next_token());
        }
    }
}
