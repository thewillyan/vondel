use enum_as_inner::EnumAsInner;

pub type TokenType = Token;

#[derive(Debug, PartialEq, Clone, EnumAsInner)]
pub enum Token {
    Ident(String),
    Integer(String),
    Illegal,
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
