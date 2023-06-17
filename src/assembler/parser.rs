use super::sections::DataKind;
use super::sections::Sections;
use std::mem::discriminant;
use std::rc::Rc;

use crate::assembler::{
    sections::{DataWrited, Instruction, TextSegment},
    tokens::{Opcode, PseudoOps},
};

use super::tokens::AsmToken;
use super::tokens::TokWithCtx;
use anyhow::{bail, Error, Result};
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum ParserError {
    #[error("Unexpected token: {tok}\nContext: line {cur_line}, column {cur_column}")]
    UnexpectedToken {
        tok: String,
        cur_line: usize,
        cur_column: usize,
    },

    #[error(
        "Expected token: {expected}, found: {found}\nContext: line {cur_line}, column {cur_column}"
    )]
    ExpectedToken {
        expected: String,
        found: String,
        cur_line: usize,
        cur_column: usize,
    },

    #[error("Expected '.byte' or '.word', found: {found}\nContext: line {cur_line}, column {cur_column}")]
    ExpectedPseudoOpsType {
        found: String,
        cur_line: usize,
        cur_column: usize,
    },

    #[error("Expected number, found: {found}\nContext: line {cur_line}, column {cur_column}")]
    ExpectedNumber {
        found: String,
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
    toks: Rc<[TokWithCtx]>,
    cur_tok: Rc<AsmToken>,
    peek_tok: Rc<AsmToken>,
    idx: usize,
    cur_line: usize,
    cur_column: usize,
}

impl Parser {
    pub fn new(toks: Rc<[TokWithCtx]>) -> Parser {
        let toks = Rc::clone(&toks);
        let mut p = Parser {
            toks,
            cur_tok: Rc::new(AsmToken::Eof),
            peek_tok: Rc::new(AsmToken::Eof),
            idx: 0,
            cur_line: 0,
            cur_column: 0,
        };

        p.next_token();
        p.next_token();

        p
    }

    fn next_token(&mut self) {
        self.cur_tok = Rc::clone(&self.peek_tok);
        if self.idx + 1 >= self.toks.len() {
            self.peek_tok = Rc::new(AsmToken::Eof);
        } else {
            self.peek_tok = Rc::clone(&self.toks[self.idx].tok);
            self.idx += 1;
        }
    }

    fn expect_peek(&mut self, expected: AsmToken) -> Result<()> {
        let disc_peek = discriminant(&(*self.peek_tok));
        let disc_expected = discriminant(&expected);
        if disc_peek == disc_expected {
            self.next_token();
        } else {
            bail!(ParserError::ExpectedToken {
                expected: format!("{:?}", disc_expected),
                found: format!("{:?}", disc_peek),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            })
        }
        Ok(())
    }

    fn parse_data_to_write(&mut self) -> Result<DataWrited> {
        let label = self.cur_tok.get_label(self.cur_line, self.cur_column)?;
        self.expect_peek(AsmToken::Colon)?;
        self.next_token();
        let res = match *self.cur_tok {
            AsmToken::PseudoOp(PseudoOps::Byte) => {
                self.expect_peek(AsmToken::Number(Rc::from("")))?;
                let number_str = self.cur_tok.get_number(self.cur_line, self.cur_column)?;
                let number = number_str.parse::<u8>()?;
                Sections::new_data_writed(DataKind::Byte(number), Rc::clone(&label))
            }
            AsmToken::PseudoOp(PseudoOps::Word) => {
                self.expect_peek(AsmToken::Number(Rc::from("")))?;
                let number_str = self.cur_tok.get_number(self.cur_line, self.cur_column)?;
                let number = number_str.parse::<i32>()?;
                Sections::new_data_writed(DataKind::Word(number), label)
            }
            _ => bail!(ParserError::ExpectedPseudoOpsType {
                found: format!("{:?}", self.cur_tok),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            }),
        };
        Ok(res)
    }

    fn parse_data_directive(&mut self) -> Result<Sections> {
        let mut data = Vec::new();
        self.next_token();

        while discriminant(&(*self.cur_tok)) == discriminant(&AsmToken::Label(Rc::from(""))) {
            data.push(self.parse_data_to_write()?);
            self.next_token();
        }

        Ok(Sections::new_data_section(data))
    }

    fn parse_global_section(&mut self) -> Result<TextSegment> {
        let mut res = Vec::new();
        self.next_token();

        while discriminant(&(*self.cur_tok)) == discriminant(&AsmToken::Label(Rc::from(""))) {
            res.push(self.cur_tok.get_label(self.cur_line, self.cur_column)?);
            self.next_token();
        }
        Ok(Sections::new_global_section(res))
    }

    fn get_instruction(&mut self) -> Result<Instruction> {
        let ins = self
            .cur_tok
            .get_instruction(self.cur_line, self.cur_column)?;
        match *ins {
            Opcode::Add => todo!(),
            _ => todo!(),
        }
        todo!()
    }

    fn parse_labeled_section(&mut self) -> Result<TextSegment> {
        let label = self.cur_tok.get_label(self.cur_line, self.cur_column)?;
        self.expect_peek(AsmToken::Colon)?;
        self.next_token();
        let mut ins = Vec::new();

        while discriminant(&(*self.cur_tok))
            == discriminant(&AsmToken::Opcode(Rc::new(Opcode::Add)))
        {
            ins.push(self.get_instruction()?);
            self.next_token();
        }

        let rc_slice: Rc<[Instruction]> = Rc::from(ins.into_boxed_slice());
        Ok(Sections::new_labeled_section(label, rc_slice))
    }

    fn parse_text_directive(&mut self) -> Result<Sections> {
        let mut data = Vec::new();
        self.next_token();
        loop {
            match *self.cur_tok {
                AsmToken::PseudoOp(PseudoOps::Global) => {
                    data.push(self.parse_global_section()?);
                }
                AsmToken::Label(_) => {
                    data.push(self.parse_labeled_section()?);
                }
                // AsmToken::PseudoOp(PseudoOps::Text) => {
                //     data.push(self.parse_text_directive()?);
                // }
                _ => break,
            }
        }

        Ok(Sections::new_text_section(data))
    }

    fn parse_pseudo_ops(&mut self) -> Result<Sections> {
        let op = (*self.cur_tok).get_pseudo_op()?;
        let res = match op {
            PseudoOps::Data => self.parse_data_directive()?,
            PseudoOps::Text => self.parse_text_directive()?,
            _ => todo!(),
        };

        Ok(res)
    }

    fn parse_shit(&mut self) -> Result<Sections> {
        let res = match *self.cur_tok {
            AsmToken::PseudoOp(_) => self.parse_pseudo_ops()?,
            AsmToken::Illegal => bail!(ParserError::UnexpectedToken {
                tok: format!("{:?}", self.cur_tok),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            }),
            _ => todo!(),
        };

        Ok(res)
    }

    pub fn get_deez_program(&mut self) -> Program {
        let mut program = Program::default();
        while *self.cur_tok != AsmToken::Eof {
            match self.parse_shit() {
                Ok(sec) => program.sections.push(sec),
                Err(e) => program.errors.push(e),
            };
            self.next_token();
        }
        program
    }
}

#[cfg(test)]
mod tests {
    use crate::assembler::lexer::Lexer;

    use super::*;

    fn create_program(input: &str) -> Program {
        let mut l = Lexer::new(input);
        let toks = l.get_deez_toks_w_ctx();
        let rc_slice = Rc::from(toks.into_boxed_slice());
        let mut p = Parser::new(rc_slice);
        p.get_deez_program()
    }

    #[test]
    fn test_data_writted() -> Result<()> {
        let input = r"
.data
    dividend:   .word 10    # Dividend
    divisor:    .word 3     # Divisor
    quotient:   .word 0     # Quotient
    remainder:  .word 0     # Remainder
    address:    .byte 77    # Address
        ";

        let program = create_program(input);

        let expected = Sections::DataSection(vec![
            Sections::new_data_writed(DataKind::Word(10), Rc::from("dividend")),
            Sections::new_data_writed(DataKind::Word(3), Rc::from("divisor")),
            Sections::new_data_writed(DataKind::Word(0), Rc::from("quotient")),
            Sections::new_data_writed(DataKind::Word(0), Rc::from("remainder")),
            Sections::new_data_writed(DataKind::Byte(77), Rc::from("address")),
        ]);

        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }
}
