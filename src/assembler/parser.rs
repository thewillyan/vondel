use super::sections::DataKind;
use super::sections::Sections;
use std::{iter::Peekable, vec::IntoIter};
use std::{mem::discriminant, slice::Iter};

use crate::assembler::{sections::DataWrited, tokens::PseudoOps};

use super::tokens::AsmToken;
use super::tokens::TokWithCtx;
use anyhow::{bail, Error, Result};
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
enum ParserError {
    #[error("Unexpected token: {tok:?}\nContext: line {cur_line}, column {cur_column}")]
    UnexpectedToken {
        tok: AsmToken,
        cur_line: usize,
        cur_column: usize,
    },

    #[error("Expected '.byte' or '.word', found: {found:?}\nContext: line {cur_line}, column {cur_column}")]
    ExpectedPseudoOpsType {
        found: AsmToken,
        cur_line: usize,
        cur_column: usize,
    },

    #[error("Expected number, found: {found:?}\nContext: line {cur_line}, column {cur_column}")]
    ExpectedNumber {
        found: AsmToken,
        cur_line: usize,
        cur_column: usize,
    },
}

#[derive(Debug, Default)]
pub struct Program {
    pub sections: Vec<Sections>,
    pub errors: Vec<Error>,
}

pub struct Parser {
    toks: Peekable<IntoIter<TokWithCtx>>,
    cur_tok: AsmToken,
    cur_line: usize,
    cur_column: usize,
}

impl Parser {
    pub fn new(toks: Vec<TokWithCtx>) -> Parser {
        let mut toks = toks.into_iter().peekable();
        let cur = toks.next().unwrap();
        let cur_tok = cur.tok;
        let cur_line = cur.cur_line;
        let cur_column = cur.cur_column;
        Parser {
            toks,
            cur_tok,
            cur_line,
            cur_column,
        }
    }

    fn next_token(&mut self) {
        self.toks.next();
    }

    fn expect_peek(&mut self, expected: AsmToken) -> Result<()> {
        let peek = self
            .toks
            .peek()
            .ok_or_else(|| ParserError::UnexpectedToken {
                tok: self.cur_tok.clone(),
                cur_line: self.cur_line,
                cur_column: self.cur_column,
            })?;
        if discriminant(&peek.tok) == discriminant(&expected) {
            self.next_token();
        } else {
            bail!(ParserError::UnexpectedToken {
                tok: self.cur_tok.clone(),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            })
        }
        Ok(())
    }

    fn parse_data_writted(&mut self) -> Result<DataWrited> {
        let label = match &self.cur_tok {
            AsmToken::Label(l) => l.clone(),
            _ => unreachable!(),
        };
        self.expect_peek(AsmToken::Colon)?;
        self.next_token();
        let res = match &self.cur_tok {
            AsmToken::PseudoOp(PseudoOps::Byte) => {
                self.expect_peek(AsmToken::Number("".to_string()))?;
                let number = match &self.cur_tok {
                    AsmToken::Number(l) => l.parse::<u8>()?,
                    _ => unreachable!(),
                };
                Sections::new_data_writed(DataKind::Byte(number), label)
            }
            AsmToken::PseudoOp(PseudoOps::Word) => {
                self.expect_peek(AsmToken::Number("".to_string()))?;
                let number = match &self.cur_tok {
                    AsmToken::Number(l) => l.parse::<i32>()?,
                    _ => unreachable!(),
                };
                Sections::new_data_writed(DataKind::Word(number), label)
            }
            _ => bail!(ParserError::ExpectedPseudoOpsType {
                found: self.cur_tok.clone(),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            }),
        };
        Ok(res)
    }

    fn parse_data_directive(&mut self) -> Result<Sections> {
        let mut data = Vec::new();
        self.next_token();

        while discriminant(&self.cur_tok) == discriminant(&AsmToken::Label("".to_string())) {
            data.push(self.parse_data_writted()?);
        }

        Ok(Sections::new_data_section(data))
    }

    fn parse_pseudo_ops(&mut self, op: PseudoOps) -> Result<Sections> {
        match op {
            PseudoOps::Data => self.parse_data_directive()?,
            _ => todo!(),
        };

        todo!()
    }

    fn parse_shit(&mut self) -> Result<Sections> {
        let res = match &self.cur_tok {
            AsmToken::PseudoOp(p) => self.parse_pseudo_ops(p.clone())?,
            AsmToken::Illegal => bail!(ParserError::UnexpectedToken {
                tok: self.cur_tok.clone(),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            }),
            _ => todo!(),
        };

        Ok(res)
    }

    pub fn get_deez_program(&mut self) {
        let mut program = Program::default();
        while self.cur_tok != AsmToken::Eof {
            match self.parse_shit() {
                Ok(sec) => program.sections.push(sec),
                Err(e) => program.errors.push(e),
            };
            self.next_token();
        }
    }
}
