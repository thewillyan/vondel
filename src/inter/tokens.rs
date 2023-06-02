use core::fmt;

use enum_as_inner::EnumAsInner;

pub type TokenType = Token;

#[derive(Debug, PartialEq, Clone, EnumAsInner)]
pub enum Token {
    Ident(String),
    Integer(String),
    Illegal(String),
    Eof,
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = match self {
            Token::Ident(v) => v,
            Token::Integer(v) => v,
            Token::Illegal(v) => v,
            Token::Eof => "",
            Token::Comma => ",",
            Token::Semicolon => ";",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::LSquirly => "{",
            Token::RSquirly => "}",
            Token::True => "true",
            Token::False => "false",
            Token::Return => "return",
            Token::If => "if",
            Token::Else => "else",
            Token::Function => "fn",
            Token::Let => "let",
            Token::Assign => "=",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Bang => "!",
            Token::Asterisk => "*",
            Token::Slash => "/",
            Token::LessThan => "<",
            Token::GreaterThan => ">",
            Token::LessEQ => "<=",
            Token::GreaterEQ => ">=",
            Token::Equal => "==",
            Token::NotEqual => "!=",
        };
        write!(f, "{}", buf)
    }
}
