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
}

pub struct Program {
    // pub statements: Vec<Statement>,
    pub errors: Vec<Error>,
}

pub struct Parser<'a> {
    cur_token: &'a TokWithCtx,
    peek_token: &'a TokWithCtx,
    idx: usize,
    toks: &'a [TokWithCtx],
}

impl<'a> Parser<'a> {
    pub fn new(toks: &'a [TokWithCtx]) -> Parser<'a> {
        let mut p = Parser {
            cur_token: &TokWithCtx {
                tok: super::tokens::AsmToken::Eof,
                cur_line: 0,
                cur_column: 0,
            },
            peek_token: &TokWithCtx {
                tok: super::tokens::AsmToken::Eof,
                cur_line: 0,
                cur_column: 0,
            },
            idx: 0,
            toks,
        };

        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token;

        if self.idx + 1 >= self.toks.len() {
            self.peek_token = &TokWithCtx {
                tok: super::tokens::AsmToken::Eof,
                cur_line: 0,
                cur_column: 0,
            };
        } else {
            self.peek_token = &self.toks[self.idx];
            self.idx += 1;
        }
    }
}
