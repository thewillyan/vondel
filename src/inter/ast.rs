use super::tokens::TokenType;
use anyhow::{bail, Error, Result};
use thiserror::Error;
mod expression;
use expression::Expression;

#[derive(Error, Debug, PartialEq)]
enum ParserError {
    #[error("Expected token '{expected}', found '{found}'")]
    MissingToken {
        expected: TokenType,
        found: TokenType,
    },

    #[error("Unable to parse integer {int:?}")]
    ParsingInteger { int: String },

    #[error("Unexpected Prefix {prefix:?}, allowed prefix are '!' '-'")]
    NotAllowedPrefix { prefix: TokenType },

    #[error("Unexpected Infix {infix:?}")]
    NotAllowedInfix { infix: TokenType },

    #[error("Unexpected Boolean {bool:?}")]
    NotAllowedBoolean { bool: TokenType },

    #[error("Illegal Token '{tok}'")]
    IllegalToken { tok: TokenType },
}

#[derive(Error, Debug)]
struct ErrorWithCtx {
    error: Error,
    ctx: String,
}

impl std::fmt::Display for ErrorWithCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} \n-> {}", self.ctx, self.error)
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum Precedence {
    Lowest = 1,
    Equals = 2,
    LessGreater = 3,
    Sum = 4,
    Product = 5,
    Prefix = 6,
    Call = 7,
}

impl Precedence {
    fn precedence_of(t: &TokenType) -> Precedence {
        // TODO: Add more precedence for LESSEQ AND GREATEREQ
        match t {
            TokenType::Equal | TokenType::NotEqual => Precedence::Equals,
            TokenType::LessThan | TokenType::GreaterThan => Precedence::LessGreater,
            TokenType::Plus | TokenType::Minus => Precedence::Sum,
            TokenType::Slash | TokenType::Asterisk => Precedence::Product,
            TokenType::LParen => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }
}

pub struct Program {
    pub statements: Vec<StatementType>,
    pub errors: Vec<Error>,
}

#[derive(Debug, PartialEq)]
pub enum StatementType {
    Let { name: Expression, value: Expression },
    Return(Expression),
    Expression(Expression),
    Block(Vec<StatementType>),
}

pub struct Parser<'a> {
    cur_token: &'a TokenType,
    peek_token: &'a TokenType,
    idx: usize,
    toks: &'a [TokenType],
}

impl<'a> Parser<'a> {
    pub fn new(toks: &'a [TokenType]) -> Parser {
        let mut p = Parser {
            cur_token: &TokenType::Eof,
            peek_token: &TokenType::Eof,
            toks,
            idx: 0,
        };
        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token;
        if self.idx + 1 >= self.toks.len() {
            self.peek_token = &TokenType::Eof;
        } else {
            self.peek_token = &self.toks[self.idx];
            self.idx += 1;
        }
    }

    fn expect_peek(&mut self, t: TokenType) -> Result<&'a TokenType> {
        let tok = match *self.peek_token {
            TokenType::Ident(_) => TokenType::Ident(String::new()),
            TokenType::Integer(_) => TokenType::Integer(String::from("0")),
            _ => self.peek_token.clone(),
        };

        if tok == t {
            self.next_token();
            Ok(self.cur_token)
        } else {
            let err = ParserError::MissingToken {
                expected: t,
                found: self.peek_token.clone(),
            }
            .into();
            bail!(self.errors_with_ctx(err, self.idx - 2))
        }
    }

    fn skip_peek_semicolon(&mut self) {
        if *self.peek_token == TokenType::Semicolon {
            self.next_token();
        }
    }

    fn curr_token_is(&self, t: TokenType) -> bool {
        *self.cur_token == t
    }

    fn peek_token_is(&self, t: TokenType) -> bool {
        *self.peek_token == t
    }

    fn peek_precedence(&self) -> Precedence {
        Precedence::precedence_of(self.peek_token)
    }

    fn cur_precedence(&self) -> Precedence {
        Precedence::precedence_of(self.cur_token)
    }

    fn parse_identifier_expression(&self, s: &String) -> Result<Expression> {
        Ok(Expression::new_ident(s))
    }

    fn parse_integer_expression(&self, s: &String) -> Result<Expression> {
        Expression::new_integer(s)
    }

    fn parse_boolean_expression(&self) -> Result<Expression> {
        Expression::new_boolean(self.cur_token)
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression> {
        let op = self.cur_token;
        self.next_token();
        let right = self.parse_expression(Precedence::Prefix)?;
        Expression::new_prefix(op, right)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression> {
        let op = self.cur_token;
        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;

        Expression::new_infix(left, op, right)
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression> {
        self.next_token();
        let exp = self.parse_expression(Precedence::Lowest)?;
        self.expect_peek(TokenType::RParen)?;

        Ok(exp)
    }

    fn parse_block_statement(&mut self) -> Result<StatementType> {
        let mut sttms = Vec::new();
        self.next_token();

        while !self.curr_token_is(TokenType::RSquirly) && !self.curr_token_is(TokenType::Eof) {
            let sttm = self.parse_statement()?;
            sttms.push(sttm);
            self.next_token();
        }

        Ok(StatementType::Block(sttms))
    }

    fn parse_if_expression(&mut self) -> Result<Expression> {
        self.expect_peek(TokenType::LParen)?;
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;
        self.expect_peek(TokenType::RParen)?;
        self.expect_peek(TokenType::LSquirly)?;

        let consequence = self.parse_block_statement()?;
        let mut alternative = None;

        if self.peek_token_is(TokenType::Else) {
            self.next_token();
            self.expect_peek(TokenType::LSquirly)?;
            alternative = Some(self.parse_block_statement()?);
        };

        Ok(Expression::new_if(condition, consequence, alternative))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Expression>> {
        let mut params = Vec::new();

        self.next_token();

        if self.curr_token_is(TokenType::RParen) {
            return Ok(params);
        }

        let ident = self.parse_identifier_expression(self.cur_token.as_ident().unwrap())?;
        params.push(ident);

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            let ident = self.parse_identifier_expression(self.cur_token.as_ident().unwrap())?;
            params.push(ident);
        }

        self.expect_peek(TokenType::RParen)?;

        Ok(params)
    }

    fn parse_function_literal(&mut self) -> Result<Expression> {
        self.expect_peek(TokenType::LParen)?;
        let params = self.parse_function_parameters()?;
        self.expect_peek(TokenType::LSquirly)?;
        let block = self.parse_block_statement()?;

        Ok(Expression::new_function(params, block))
    }

    fn prefix_parse_fns(&mut self) -> Option<Result<Expression>> {
        match self.cur_token {
            TokenType::Ident(v) => Some(self.parse_identifier_expression(v)),
            TokenType::Integer(v) => Some(self.parse_integer_expression(v)),
            TokenType::Bang | TokenType::Minus => Some(self.parse_prefix_expression()),
            TokenType::True | TokenType::False => Some(self.parse_boolean_expression()),
            TokenType::LParen => Some(self.parse_grouped_expression()),
            TokenType::If => Some(self.parse_if_expression()),
            TokenType::Function => Some(self.parse_function_literal()),
            _ => None,
        }
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();

        self.next_token();

        if self.curr_token_is(TokenType::RParen) {
            return Ok(args);
        }

        args.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(TokenType::Comma) {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.expect_peek(TokenType::RParen)?;

        Ok(args)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression> {
        let args = self.parse_call_arguments()?;
        Ok(Expression::new_call(function, args))
    }

    fn infix_parse_fns(
        &mut self,
    ) -> Option<Box<dyn FnOnce(Expression) -> Result<Expression> + '_>> {
        match *self.peek_token {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Slash
            | TokenType::Asterisk
            | TokenType::Equal
            | TokenType::NotEqual
            | TokenType::LessThan
            | TokenType::GreaterThan => {
                self.next_token();
                Some(Box::new(|v| self.parse_infix_expression(v)))
            }
            TokenType::LParen => {
                self.next_token();
                Some(Box::new(|v| self.parse_call_expression(v)))
            }
            _ => None,
        }
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        let prefix = self
            .prefix_parse_fns()
            .unwrap_or_else(|| bail!("no prefix parse function for {:?}", self.cur_token))?;

        let mut left_exp = prefix;

        while !self.peek_token_is(TokenType::Semicolon) && precedence < self.peek_precedence() {
            match self.infix_parse_fns() {
                Some(infix) => {
                    left_exp = infix(left_exp)?;
                }
                None => return Ok(left_exp),
            }
        }

        Ok(left_exp)
    }

    fn parse_let_statement(&mut self) -> Result<StatementType> {
        let res = self
            .expect_peek(TokenType::Ident(String::new()))?
            .as_ident()
            .unwrap();
        let name = Expression::Identifier(res.to_string());

        self.expect_peek(TokenType::Assign)?;
        self.next_token();

        let value = self.parse_expression(Precedence::Lowest)?;
        self.skip_peek_semicolon();

        Ok(StatementType::Let { name, value })
    }

    fn parse_return_statement(&mut self) -> Result<StatementType> {
        self.next_token();
        let ret_value = self.parse_expression(Precedence::Lowest)?;

        self.skip_peek_semicolon();

        Ok(StatementType::Return(ret_value))
    }

    fn parse_expression_statement(&mut self) -> Result<StatementType> {
        let res = self.parse_expression(Precedence::Lowest)?;
        self.skip_peek_semicolon();
        Ok(StatementType::Expression(res))
    }

    fn parse_statement(&mut self) -> Result<StatementType> {
        let res = match self.cur_token {
            TokenType::Let => self.parse_let_statement()?,
            TokenType::Return => self.parse_return_statement()?,
            TokenType::Illegal(_) => bail!(ParserError::IllegalToken {
                tok: self.cur_token.clone()
            }),
            _ => self.parse_expression_statement()?,
        };

        Ok(res)
    }

    fn errors_with_ctx(&mut self, error: anyhow::Error, idx: usize) -> anyhow::Error {
        let mut ctx = String::new();
        let mut i = idx;

        while i != 0 && self.toks[i] != TokenType::Semicolon {
            ctx.insert_str(0, &format!("{} ", self.toks[i]));
            i -= 1;
        }

        if i == 0 {
            ctx.insert_str(0, &format!("{} ", self.toks[i]));
        }

        while self.peek_token != &TokenType::Eof && self.peek_token != &TokenType::Semicolon {
            ctx.push_str(&format!("{}", self.peek_token));
            self.next_token();
        }

        self.next_token();

        ErrorWithCtx { error, ctx }.into()
    }

    pub fn get_deez_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
            errors: Vec::new(),
        };

        while *self.cur_token != TokenType::Eof {
            match self.parse_statement() {
                Ok(stmt) => program.statements.push(stmt),
                Err(e) => program.errors.push(e),
            }
            self.next_token();
        }

        program
    }
}

#[cfg(test)]
mod tests {
    use crate::inter::ast::expression::{InfixOp, PrefixOp};

    use super::*;
    use anyhow::anyhow;

    #[test]
    fn parse_let_statements() {
        /* let tubias = x;
        let foobar = y;
        let barfoo  z;
        // Error above
        */
        let toks = vec![
            TokenType::Let,
            TokenType::Ident(String::from("tubias")),
            TokenType::Assign,
            TokenType::Ident(String::from("x")),
            TokenType::Semicolon,
            TokenType::Let,
            TokenType::Ident(String::from("foobar")),
            TokenType::Assign,
            TokenType::Ident(String::from("y")),
            TokenType::Semicolon,
            TokenType::Let,
            TokenType::Ident(String::from("barfoo")),
            TokenType::Ident(String::from("z")),
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        let stts = vec![
            StatementType::Let {
                name: Expression::new_ident(&String::from("tubias")),
                value: Expression::new_ident(&String::from("x")),
            },
            StatementType::Let {
                name: Expression::new_ident(&String::from("foobar")),
                value: Expression::new_ident(&String::from("y")),
            },
        ];
        let err = anyhow!(ErrorWithCtx {
            error: anyhow!(ParserError::MissingToken {
                expected: TokenType::Assign,
                found: TokenType::Ident("z".to_string())
            }),
            ctx: String::from("let barfoo z"),
        });

        assert_eq!(program.errors.len(), 1);
        assert_eq!(program.statements.len(), 2);

        assert_eq!(program.errors[0].to_string(), err.to_string());

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn parse_return_statements() {
        /* return 5;
         * return 10; */
        let toks = vec![
            TokenType::Return,
            TokenType::Integer(String::from("5")),
            TokenType::Semicolon,
            TokenType::Return,
            TokenType::Integer(String::from("10")),
            TokenType::Semicolon,
            TokenType::Eof,
        ];
        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        let stts = vec![
            StatementType::Return(Expression::new_integer(&"5".to_string()).unwrap()),
            StatementType::Return(Expression::new_integer(&"10".to_string()).unwrap()),
        ];

        assert_eq!(program.statements.len(), 2);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn parse_identifier_expression() {
        //foobar;
        let toks = vec![
            TokenType::Ident(String::from("foobar")),
            TokenType::Semicolon,
            TokenType::Eof,
        ];
        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        let stts = vec![StatementType::Expression(Expression::Identifier(
            String::from("foobar"),
        ))];

        assert_eq!(program.statements.len(), 1);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn parse_integer_expression() {
        //foobar;
        let toks = vec![
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Eof,
        ];
        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        let stts = vec![StatementType::Expression(Expression::Integer(5))];

        assert_eq!(program.statements.len(), 1);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn parse_prefix_operator() {
        /* !5;
         *tubias;
         * !false;
         * !true;
         */
        let toks = vec![
            TokenType::Bang,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Minus,
            TokenType::Ident(String::from("tubias")),
            TokenType::Semicolon,
            TokenType::Bang,
            TokenType::False,
            TokenType::Semicolon,
            TokenType::Bang,
            TokenType::True,
            TokenType::Semicolon,
            TokenType::Eof,
        ];
        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        let stts = vec![
            StatementType::Expression(Expression::Prefix {
                op: PrefixOp::Bang,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Prefix {
                op: PrefixOp::Minus,
                right: Box::new(Expression::Identifier(String::from("tubias"))),
            }),
            StatementType::Expression(Expression::Prefix {
                op: PrefixOp::Bang,
                right: Box::new(Expression::Boolean(false)),
            }),
            StatementType::Expression(Expression::Prefix {
                op: PrefixOp::Bang,
                right: Box::new(Expression::Boolean(true)),
            }),
        ];

        assert_eq!(program.statements.len(), 4);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn parse_infix_expressions() {
        /*
         * 5 + 5;
         * 5 - 5;
         * 5 * 5;
         * 5 / 5;
         * 5 > 5;
         * 5 < 5;
         * 5 == 5;
         * 5 != 5;
         *  false != true;
         *  true == true;
         */
        let toks = vec![
            TokenType::Integer(5.to_string()),
            TokenType::Plus,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Integer(5.to_string()),
            TokenType::Minus,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Integer(5.to_string()),
            TokenType::Asterisk,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Integer(5.to_string()),
            TokenType::Slash,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Integer(5.to_string()),
            TokenType::GreaterThan,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Integer(5.to_string()),
            TokenType::LessThan,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Integer(5.to_string()),
            TokenType::Equal,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Integer(5.to_string()),
            TokenType::NotEqual,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::False,
            TokenType::NotEqual,
            TokenType::True,
            TokenType::Semicolon,
            TokenType::True,
            TokenType::Equal,
            TokenType::True,
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        let stts = vec![
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::Plus,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::Minus,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::Asterisk,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::Slash,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::GreaterThan,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::LessThan,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::Equal,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::NotEqual,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Boolean(false)),
                op: InfixOp::NotEqual,
                right: Box::new(Expression::Boolean(true)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Boolean(true)),
                op: InfixOp::Equal,
                right: Box::new(Expression::Boolean(true)),
            }),
        ];

        assert_eq!(program.statements.len(), 10);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn test_operator_precedence() {
        /*
         * a + b * c + d / e - f;
         * 3 + 4 * 5 == 3 * 1 + 4 * 5;
         * 3 > 5 == false;
         * (10 + (5 * 7)) / 3;
         * --------------------------
         * RES
         * --------------------------
         * ((a + (b * c)) + (d / e)) - f
         * ((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))
         * ((3 > 5) == false)
         * ((10 + (5 * 7)) / 3)
         * */

        let toks = vec![
            TokenType::Ident(String::from("a")),
            TokenType::Plus,
            TokenType::Ident(String::from("b")),
            TokenType::Asterisk,
            TokenType::Ident(String::from("c")),
            TokenType::Plus,
            TokenType::Ident(String::from("d")),
            TokenType::Slash,
            TokenType::Ident(String::from("e")),
            TokenType::Minus,
            TokenType::Ident(String::from("f")),
            TokenType::Semicolon,
            TokenType::Integer(3.to_string()),
            TokenType::Plus,
            TokenType::Integer(4.to_string()),
            TokenType::Asterisk,
            TokenType::Integer(5.to_string()),
            TokenType::Equal,
            TokenType::Integer(3.to_string()),
            TokenType::Asterisk,
            TokenType::Integer(1.to_string()),
            TokenType::Plus,
            TokenType::Integer(4.to_string()),
            TokenType::Asterisk,
            TokenType::Integer(5.to_string()),
            TokenType::Semicolon,
            TokenType::Integer(3.to_string()),
            TokenType::GreaterThan,
            TokenType::Integer(5.to_string()),
            TokenType::Equal,
            TokenType::False,
            TokenType::Semicolon,
            TokenType::LParen,
            TokenType::Integer(10.to_string()),
            TokenType::Plus,
            TokenType::LParen,
            TokenType::Integer(5.to_string()),
            TokenType::Asterisk,
            TokenType::Integer(7.to_string()),
            TokenType::RParen,
            TokenType::RParen,
            TokenType::Slash,
            TokenType::Integer(3.to_string()),
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        let stts = vec![
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::Identifier(String::from("a"))),
                        op: InfixOp::Plus,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Identifier(String::from("b"))),
                            op: InfixOp::Asterisk,
                            right: Box::new(Expression::Identifier(String::from("c"))),
                        }),
                    }),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Identifier(String::from("d"))),
                        op: InfixOp::Slash,
                        right: Box::new(Expression::Identifier(String::from("e"))),
                    }),
                }),
                op: InfixOp::Minus,
                right: Box::new(Expression::Identifier(String::from("f"))),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(3)),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer(4)),
                        op: InfixOp::Asterisk,
                        right: Box::new(Expression::Integer(5)),
                    }),
                }),
                op: InfixOp::Equal,
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer(3)),
                        op: InfixOp::Asterisk,
                        right: Box::new(Expression::Integer(1)),
                    }),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer(4)),
                        op: InfixOp::Asterisk,
                        right: Box::new(Expression::Integer(5)),
                    }),
                }),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(3)),
                    op: InfixOp::GreaterThan,
                    right: Box::new(Expression::Integer(5)),
                }),
                op: InfixOp::Equal,
                right: Box::new(Expression::Boolean(false)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(10)),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer(5)),
                        op: InfixOp::Asterisk,
                        right: Box::new(Expression::Integer(7)),
                    }),
                }),
                op: InfixOp::Slash,
                right: Box::new(Expression::Integer(3)),
            }),
        ];

        assert_eq!(program.statements.len(), 4);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn parse_boolean_expression() {
        /* true;
         * false;
         * */
        let toks = vec![
            TokenType::True,
            TokenType::Semicolon,
            TokenType::False,
            TokenType::Semicolon,
        ];

        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        let stts = vec![
            StatementType::Expression(Expression::Boolean(true)),
            StatementType::Expression(Expression::Boolean(false)),
        ];

        assert_eq!(program.statements.len(), 2);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn test_if_statements() {
        /*
         * if (x < y) { x } else {y};
         * if (x) { y };
         * */
        let toks = vec![
            TokenType::If,
            TokenType::LParen,
            TokenType::Ident(String::from("x")),
            TokenType::LessThan,
            TokenType::Ident(String::from("y")),
            TokenType::RParen,
            TokenType::LSquirly,
            TokenType::Ident(String::from("x")),
            TokenType::RSquirly,
            TokenType::Else,
            TokenType::LSquirly,
            TokenType::Ident(String::from("y")),
            TokenType::RSquirly,
            TokenType::Semicolon,
            TokenType::If,
            TokenType::LParen,
            TokenType::Ident(String::from("x")),
            TokenType::RParen,
            TokenType::LSquirly,
            TokenType::Ident(String::from("y")),
            TokenType::RSquirly,
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        let stts = vec![
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    op: InfixOp::LessThan,
                    right: Box::new(Expression::Identifier("y".to_string())),
                }),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Identifier("x".to_string()),
                )])),
                alternative: Some(Box::new(StatementType::Block(vec![
                    StatementType::Expression(Expression::Identifier("y".to_string())),
                ]))),
            }),
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Identifier("x".to_string())),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Identifier("y".to_string()),
                )])),
                alternative: None,
            }),
        ];

        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        assert_eq!(program.statements.len(), 2);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn test_fn_literal() {
        /* fn(x , y , z) { x + y + z; };
         * fn(x, y) { x + y; };
         * fn() { x + y; };
         * */
        let toks = vec![
            TokenType::Function,
            TokenType::LParen,
            TokenType::Ident(String::from("x")),
            TokenType::Comma,
            TokenType::Ident(String::from("y")),
            TokenType::Comma,
            TokenType::Ident(String::from("z")),
            TokenType::RParen,
            TokenType::LSquirly,
            TokenType::Ident(String::from("x")),
            TokenType::Plus,
            TokenType::Ident(String::from("y")),
            TokenType::Plus,
            TokenType::Ident(String::from("z")),
            TokenType::Semicolon,
            TokenType::RSquirly,
            TokenType::Semicolon,
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
            TokenType::Function,
            TokenType::LParen,
            TokenType::RParen,
            TokenType::LSquirly,
            TokenType::Ident(String::from("x")),
            TokenType::Plus,
            TokenType::Ident(String::from("y")),
            TokenType::Semicolon,
            TokenType::RSquirly,
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        let stts = vec![
            StatementType::Expression(Expression::FunctionLiteral {
                parameters: vec![
                    Expression::Identifier("x".to_string()),
                    Expression::Identifier("y".to_string()),
                    Expression::Identifier("z".to_string()),
                ],
                block: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Infix {
                        left: Box::new(Expression::Infix {
                            left: Box::new(Expression::Identifier("x".to_string())),
                            op: InfixOp::Plus,
                            right: Box::new(Expression::Identifier("y".to_string())),
                        }),
                        op: InfixOp::Plus,
                        right: Box::new(Expression::Identifier("z".to_string())),
                    },
                )])),
            }),
            StatementType::Expression(Expression::FunctionLiteral {
                parameters: vec![
                    Expression::Identifier("x".to_string()),
                    Expression::Identifier("y".to_string()),
                ],
                block: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Infix {
                        left: Box::new(Expression::Identifier("x".to_string())),
                        op: InfixOp::Plus,
                        right: Box::new(Expression::Identifier("y".to_string())),
                    },
                )])),
            }),
            StatementType::Expression(Expression::FunctionLiteral {
                parameters: vec![],
                block: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Infix {
                        left: Box::new(Expression::Identifier("x".to_string())),
                        op: InfixOp::Plus,
                        right: Box::new(Expression::Identifier("y".to_string())),
                    },
                )])),
            }),
        ];

        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        assert_eq!(program.statements.len(), 3);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }

    #[test]
    fn parse_call_expressions() {
        // add(1, 2 * 3, 4 + 5);
        let toks = vec![
            TokenType::Ident(String::from("add")),
            TokenType::LParen,
            TokenType::Integer(1.to_string()),
            TokenType::Comma,
            TokenType::Integer(2.to_string()),
            TokenType::Asterisk,
            TokenType::Integer(3.to_string()),
            TokenType::Comma,
            TokenType::Integer(4.to_string()),
            TokenType::Plus,
            TokenType::Integer(5.to_string()),
            TokenType::RParen,
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        let stts = vec![StatementType::Expression(Expression::Call {
            function: Box::new(Expression::Identifier("add".to_string())),
            arguments: vec![
                Expression::Integer(1),
                Expression::Infix {
                    left: Box::new(Expression::Integer(2)),
                    op: InfixOp::Asterisk,
                    right: Box::new(Expression::Integer(3)),
                },
                Expression::Infix {
                    left: Box::new(Expression::Integer(4)),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Integer(5)),
                },
            ],
        })];

        let mut parser = Parser::new(&toks);
        let program = parser.get_deez_program();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(program.errors.len(), 0);

        for (i, s) in stts.into_iter().enumerate() {
            assert_eq!(s, program.statements[i]);
        }
    }
}
