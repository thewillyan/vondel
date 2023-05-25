use anyhow::{bail, Result};

use super::StatementType;
use crate::inter::lexer::TokenType;

#[derive(Debug, PartialEq)]
pub enum PrefixOp {
    Bang,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum InfixOp {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    Integer(i64),
    Prefix {
        op: PrefixOp,
        right: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        op: InfixOp,
        right: Box<Expression>,
    },
    Boolean(bool),
    If {
        condition: Box<Expression>,
        consequence: Box<StatementType>,
        alternative: Option<Box<StatementType>>,
    },
    FunctionLiteral {
        parameters: Vec<Expression>,
        block: Box<StatementType>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

impl Expression {
    pub fn new_ident(ident: &String) -> Self {
        Expression::Identifier(String::from(ident))
    }

    pub fn new_integer(int: &String) -> Result<Self> {
        match int.parse::<i64>() {
            Ok(v) => Ok(Expression::Integer(v)),
            Err(_) => bail!(super::ParserError::ParsingInteger {
                int: int.to_string()
            }),
        }
    }

    pub fn new_prefix(op: &TokenType, exp: Expression) -> Result<Self> {
        let op = match op {
            TokenType::Bang => PrefixOp::Bang,
            TokenType::Minus => PrefixOp::Minus,
            _ => bail!(super::ParserError::NotAllowedPrefix { prefix: op.clone() }),
        };

        Ok(Expression::Prefix {
            op,
            right: Box::new(exp),
        })
    }

    pub fn new_infix(left: Expression, op: &TokenType, right: Expression) -> Result<Self> {
        let op = match op {
            TokenType::Plus => InfixOp::Plus,
            TokenType::Minus => InfixOp::Minus,
            TokenType::Asterisk => InfixOp::Asterisk,
            TokenType::Slash => InfixOp::Slash,
            TokenType::Equal => InfixOp::Equal,
            TokenType::NotEqual => InfixOp::NotEqual,
            TokenType::LessThan => InfixOp::LessThan,
            TokenType::GreaterThan => InfixOp::GreaterThan,
            _ => bail!(super::ParserError::NotAllowedInfix { infix: op.clone() }),
        };

        Ok(Expression::Infix {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    pub fn new_boolean(t: &TokenType) -> Result<Self> {
        match *t {
            TokenType::True => Ok(Expression::Boolean(true)),
            TokenType::False => Ok(Expression::Boolean(false)),
            _ => bail!(super::ParserError::NotAllowedBoolean { bool: t.clone() }),
        }
    }

    pub fn new_if(cond: Expression, cons: StatementType, alt: Option<StatementType>) -> Self {
        Expression::If {
            condition: Box::new(cond),
            consequence: Box::new(cons),
            alternative: alt.map(Box::new),
        }
    }

    pub fn new_function(params: Vec<Expression>, block: StatementType) -> Self {
        Expression::FunctionLiteral {
            parameters: params,
            block: Box::new(block),
        }
    }

    pub fn new_call(func: Expression, args: Vec<Expression>) -> Self {
        Expression::Call {
            function: Box::new(func),
            arguments: args,
        }
    }
}
