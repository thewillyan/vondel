use super::sections::DataKind;
use super::sections::Sections;
use std::mem::discriminant;
use std::rc::Rc;

use crate::assembler::{
    sections::{BranchOp, DataWrited, ImmediateOrLabel, Instruction, TextSegment, Value},
    tokens::{Opcode, PseudoOps, Register},
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
    ExpectedByteOrWordType {
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

    #[error(
        "Expected to be in section, found: {found}\nContext: line {cur_line}, column {cur_column}"
    )]
    ExpectedToBeInSection {
        found: String,
        cur_line: usize,
        cur_column: usize,
    },

    #[error("Register cannot be used in A Bus, found: {found}\nContext: line {cur_line}, column {cur_column}")]
    RegisterCannotBeUsedInABus {
        found: String,
        cur_line: usize,
        cur_column: usize,
    },

    #[error("Register cannot be used in B Bus, found: {found}\nContext: line {cur_line}, column {cur_column}")]
    RegisterCannotBeUsedInBBus {
        found: String,
        cur_line: usize,
        cur_column: usize,
    },

    #[error("Register cannot be used in C Bus, found: {found}\nContext: line {cur_line}, column {cur_column}")]
    RegisterCannotBeUsedInCBus {
        found: String,
        cur_line: usize,
        cur_column: usize,
    },

    #[error("Operation only accepts an 'Immediate' or a 'Label'")]
    OnlyAcceptsImmediateOrLabel {
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
        self.cur_line = self.toks[self.idx].cur_line;
        self.cur_column = self.toks[self.idx].cur_column;
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
                expected: format!("{:?}", expected),
                found: format!("{:?}", self.peek_tok),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            })
        }
        Ok(())
    }

    fn curr_token_is(&mut self, expected: AsmToken) -> bool {
        let disc_curr = discriminant(&(*self.cur_tok));
        let disc_expected = discriminant(&expected);
        disc_curr == disc_expected
    }

    fn peek_token_is(&mut self, expected: AsmToken) -> bool {
        let disc_peek = discriminant(&(*self.peek_tok));
        let disc_expected = discriminant(&expected);
        disc_peek == disc_expected
    }

    fn get_label(&mut self) -> Result<Rc<str>> {
        let label = match *self.cur_tok {
            AsmToken::Label(ref l) => Rc::clone(l),
            _ => {
                bail!(ParserError::ExpectedToken {
                    expected: format!("{:?}", AsmToken::Label(Rc::from(""))),
                    found: format!("{:?}", self.cur_tok),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
        };
        Ok(label)
    }

    fn get_number(&mut self) -> Result<Rc<str>> {
        let number = match *self.cur_tok {
            AsmToken::Number(ref n) => Rc::clone(n),
            _ => {
                bail!(ParserError::ExpectedToken {
                    expected: format!("{:?}", AsmToken::Number(Rc::from(""))),
                    found: format!("{:?}", self.cur_tok),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
        };
        Ok(number)
    }

    fn get_register(&self) -> Result<Rc<Register>> {
        let reg = match *self.cur_tok {
            AsmToken::Reg(ref r) => Rc::clone(r),
            _ => {
                bail!(ParserError::ExpectedToken {
                    expected: format!("{:?}", "Register"),
                    found: format!("{:?}", self.cur_tok),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
        };
        Ok(reg)
    }

    fn get_opcode(&mut self) -> Result<Rc<Opcode>> {
        let instr = match *self.cur_tok {
            AsmToken::Opcode(ref o) => Rc::clone(o),
            _ => {
                bail!(ParserError::ExpectedToken {
                    expected: format!("{:?}", "Opcode"),
                    found: format!("{:?}", self.cur_tok),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
        };
        Ok(instr)
    }

    fn op_to_branch_op(&mut self, op: Rc<Opcode>) -> Result<BranchOp> {
        let res = match *op {
            Opcode::Beq => BranchOp::Beq,
            Opcode::Bne => BranchOp::Bne,
            Opcode::Blt => BranchOp::Blt,
            Opcode::Bgt => BranchOp::Bgt,
            _ => {
                bail!(ParserError::ExpectedToken {
                    expected: format!("{:?}", "BranchOp"),
                    found: format!("{:?}", op),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
        };

        Ok(res)
    }

    fn get_pseudo_op(&mut self) -> Result<Rc<PseudoOps>> {
        let pseudo_op = match *self.cur_tok {
            AsmToken::PseudoOp(ref p) => Rc::clone(p),
            _ => {
                bail!(ParserError::ExpectedToken {
                    expected: format!("{:?}", "PseudoOp"),
                    found: format!("{:?}", self.cur_tok),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
        };
        Ok(pseudo_op)
    }

    fn guard_a_bus(&mut self, reg: Rc<Register>) -> Result<Rc<Register>> {
        match *reg {
            Register::Mar => {
                bail!(ParserError::RegisterCannotBeUsedInABus {
                    found: format!("{:?}", reg),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
            _ => Ok(reg),
        }
    }

    fn guard_b_bus(&mut self, reg: Rc<Register>) -> Result<Rc<Register>> {
        match *reg {
            Register::Mar
            | Register::Mbr
            | Register::Mbr2
            | Register::Pc
            | Register::Mbru
            | Register::Mbr2u => {
                bail!(ParserError::RegisterCannotBeUsedInBBus {
                    found: format!("{:?}", reg),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
            _ => Ok(reg),
        }
    }

    fn guard_c_bus(&self, reg: Rc<Register>) -> Result<Rc<Register>> {
        match *reg {
            Register::Cpp | Register::Mbr | Register::Mbr2 | Register::Mbru | Register::Mbr2u => {
                bail!(ParserError::RegisterCannotBeUsedInCBus {
                    found: format!("{:?}", reg),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
            _ => Ok(reg),
        }
    }

    fn parse_data_to_write(&mut self) -> Result<DataWrited> {
        let label = self.get_label()?;
        self.expect_peek(AsmToken::Colon)?;
        self.next_token();
        let op = self.get_pseudo_op()?;
        let res = match *op {
            PseudoOps::Byte => {
                self.expect_peek(AsmToken::Number(Rc::from("")))?;
                let number_str = self.get_number()?;
                let number = number_str.parse::<u8>()?;
                Sections::new_data_writed(DataKind::Byte(number), Rc::clone(&label))
            }
            PseudoOps::Word => {
                self.expect_peek(AsmToken::Number(Rc::from("")))?;
                let number_str = self.get_number()?;
                let number = number_str.parse::<i32>()?;
                Sections::new_data_writed(DataKind::Word(number), label)
            }
            _ => bail!(ParserError::ExpectedByteOrWordType {
                found: format!("{:?}", self.cur_tok),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            }),
        };
        Ok(res)
    }

    fn parse_data_directive(&mut self) -> Result<Sections> {
        let mut data = Vec::new();
        while discriminant(&(*self.peek_tok)) == discriminant(&AsmToken::Label(Rc::from(""))) {
            self.next_token();
            data.push(self.parse_data_to_write()?);
        }

        Ok(Sections::new_data_section(data))
    }

    fn get_dest_regs(&mut self) -> Result<Vec<Rc<Register>>> {
        let mut dest_regs = Vec::new();

        dest_regs.push(self.guard_c_bus(self.get_register()?)?);
        while self.peek_token_is(AsmToken::Comma) {
            self.next_token();
            self.next_token();
            dest_regs.push(self.guard_c_bus(self.get_register()?)?);
        }

        Ok(dest_regs)
    }

    fn parse_instruction_til_rs1(&mut self) -> Result<(Vec<Rc<Register>>, Rc<Register>)> {
        let dest_regs = self.get_dest_regs()?;
        self.expect_peek(AsmToken::Assign)?;
        self.next_token();
        let rs1 = self.guard_a_bus(self.get_register()?)?;
        Ok((dest_regs, rs1))
    }

    fn get_instruction(&mut self) -> Result<Instruction> {
        let op = self.get_opcode()?;

        let res = match *op {
            // Register Register Instructions
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::And | Opcode::Or => {
                self.next_token();
                let (dest_regs, rs1) = self.parse_instruction_til_rs1()?;
                self.expect_peek(AsmToken::Comma)?;
                self.next_token();
                let rs2 = Value::Reg(self.guard_b_bus(self.get_register()?)?);
                Instruction::new_double_operand_instruction(op, dest_regs, rs1, rs2)
            }
            // Imediate Register Instructions
            Opcode::Addi | Opcode::Andi | Opcode::Ori | Opcode::Subi => {
                self.next_token();
                let (dest_regs, rs1) = self.parse_instruction_til_rs1()?;
                self.expect_peek(AsmToken::Comma)?;
                self.next_token();
                let immediate = self.get_number()?.parse::<u8>()?;
                let rs2 = Value::Immediate(immediate);
                Instruction::new_double_operand_instruction(op, dest_regs, rs1, rs2)
            }
            Opcode::Lui => {
                self.next_token();
                let dest_regs = self.get_dest_regs()?;
                self.expect_peek(AsmToken::Assign)?;
                self.next_token();
                let immediate = self.get_number()?.parse::<u8>()?;
                let rs1 = Value::Immediate(immediate);
                Instruction::new_single_operand_instruction(op, dest_regs, rs1)
            }
            // Single Operand Instructions
            Opcode::Not | Opcode::Sll | Opcode::Sra | Opcode::Sla | Opcode::Mov => {
                self.next_token();
                let (dest_regs, rs1) = self.parse_instruction_til_rs1()?;
                let rs1 = Value::Reg(rs1);
                Instruction::new_single_operand_instruction(op, dest_regs, rs1)
            }
            // Branch motherfucker
            Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bgt => {
                self.next_token();
                let rs1 = self.guard_a_bus(self.get_register()?)?;
                self.expect_peek(AsmToken::Comma)?;
                self.next_token();
                let rs2 = self.guard_b_bus(self.get_register()?)?;
                self.expect_peek(AsmToken::Comma)?;
                self.next_token();
                let label = self.get_label()?;
                Instruction::new_branch_instruction(self.op_to_branch_op(op)?, rs1, rs2, label)
            }
            // Jal
            Opcode::Jal => {
                self.next_token();
                let label = self.get_label()?;
                Instruction::new_jal_instruction(label)
            }
            Opcode::Read => {
                self.next_token();
                let rds = self.get_dest_regs()?;
                self.expect_peek(AsmToken::Assign)?;
                self.next_token();
                let addr = match *self.cur_tok {
                    AsmToken::Number(ref n) => ImmediateOrLabel::Immediate(Rc::clone(n).parse()?),
                    AsmToken::Label(ref l) => ImmediateOrLabel::Label(Rc::clone(l)),
                    _ => bail!(ParserError::OnlyAcceptsImmediateOrLabel {
                        found: format!("{:?}", self.cur_tok),
                        cur_line: self.cur_line,
                        cur_column: self.cur_column
                    }),
                };
                Instruction::new_read_instruction(addr, rds)
            }
            Opcode::Write => {
                self.next_token();
                let addr = match *self.cur_tok {
                    AsmToken::Number(ref n) => ImmediateOrLabel::Immediate(Rc::clone(n).parse()?),
                    AsmToken::Label(ref l) => ImmediateOrLabel::Label(Rc::clone(l)),
                    _ => bail!(ParserError::OnlyAcceptsImmediateOrLabel {
                        found: format!("{:?}", self.cur_tok),
                        cur_line: self.cur_line,
                        cur_column: self.cur_column
                    }),
                };
                self.expect_peek(AsmToken::Assign)?;
                self.next_token();
                let rd = self.guard_a_bus(self.get_register()?)?;
                Instruction::new_write_instruction(addr, rd)
            }
            // No Operand Instructions
            Opcode::Halt | Opcode::Nop => Instruction::new_no_operand_instruction(op),
        };

        Ok(res)
    }

    fn parse_labeled_section(&mut self) -> Result<TextSegment> {
        let label = self.get_label()?;
        self.expect_peek(AsmToken::Colon)?;
        let mut ins = Vec::new();

        while discriminant(&(*self.peek_tok))
            == discriminant(&AsmToken::Opcode(Rc::new(Opcode::Add)))
        {
            self.next_token();
            ins.push(self.get_instruction()?);
        }
        self.next_token();

        Ok(TextSegment::new_labeled_section(label, ins))
    }

    fn parse_text_directive(&mut self) -> Result<Sections> {
        let mut data = Vec::new();
        self.next_token();
        loop {
            match *self.cur_tok {
                AsmToken::PseudoOp(ref v) => {
                    let v = Rc::clone(v);
                    match *v {
                        PseudoOps::Global => {
                            self.next_token();
                            data.push(TextSegment::new_global_section(self.get_label()?));
                            self.next_token();
                        }
                        _ => break,
                    }
                }
                AsmToken::Label(_) => {
                    data.push(self.parse_labeled_section()?);
                }
                _ => break,
            }
        }

        Ok(Sections::new_text_section(data))
    }

    fn parse_pseudo_ops(&mut self) -> Result<Sections> {
        let op = self.get_pseudo_op()?;
        let res = match *op {
            PseudoOps::Data => self.parse_data_directive()?,
            PseudoOps::Text => self.parse_text_directive()?,
            _ => {
                let tok = Rc::clone(&op);
                while discriminant(&(*self.cur_tok))
                    != discriminant(&AsmToken::PseudoOp(Rc::from(PseudoOps::Text)))
                {
                    self.next_token();
                }
                bail!(ParserError::ExpectedToBeInSection {
                    found: format!("{:?}", tok),
                    cur_line: self.cur_line,
                    cur_column: self.cur_column
                })
            }
        };

        Ok(res)
    }

    fn parse_shit(&mut self) -> Result<Sections> {
        let res = match *self.cur_tok {
            AsmToken::Illegal => bail!(ParserError::UnexpectedToken {
                tok: format!("{:?}", self.cur_tok),
                cur_line: self.cur_line,
                cur_column: self.cur_column
            }),
            _ => self.parse_pseudo_ops()?,
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

    #[test]
    fn parse_global_section() -> Result<()> {
        let input = r"
.text
.global _start
.global main
.global tubias
        ";

        let program = create_program(input);

        let expected = Sections::TextSection(vec![
            TextSegment::new_global_section(Rc::from("_start")),
            TextSegment::new_global_section(Rc::from("main")),
            TextSegment::new_global_section(Rc::from("tubias")),
        ]);

        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_double_operand_rr_instruction() -> Result<()> {
        use crate::assembler::tokens::Opcode::*;
        use crate::assembler::tokens::Register::*;
        let input = r"
.text
main:
    add t0 <- t1, t2
    sub t1, t2, t3, s0 <- t1, t2
    mul t1,t2,t3,s0,s1,s2,s3,s4,a0,a1,a2 <- a0, a1
    and t0 <- a0, a1
    or t0 <- a0, a1

.text
error:
    add t0 <- t1

.text
error2:
    add <- t1, t2
        ";

        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![
                Instruction::new_double_operand_instruction(
                    Rc::new(Add),
                    vec![Rc::from(T0)],
                    Rc::from(T1),
                    Value::Reg(Rc::from(T2)),
                ),
                Instruction::new_double_operand_instruction(
                    Rc::new(Sub),
                    vec![Rc::from(T1), Rc::from(T2), Rc::from(T3), Rc::from(S0)],
                    Rc::from(T1),
                    Value::Reg(Rc::from(T2)),
                ),
                Instruction::new_double_operand_instruction(
                    Rc::new(Mul),
                    vec![
                        Rc::from(T1),
                        Rc::from(T2),
                        Rc::from(T3),
                        Rc::from(S0),
                        Rc::from(S1),
                        Rc::from(S2),
                        Rc::from(S3),
                        Rc::from(S4),
                        Rc::from(A0),
                        Rc::from(A1),
                        Rc::from(A2),
                    ],
                    Rc::from(A0),
                    Value::Reg(Rc::from(A1)),
                ),
                Instruction::new_double_operand_instruction(
                    Rc::new(And),
                    vec![Rc::from(T0)],
                    Rc::from(A0),
                    Value::Reg(Rc::from(A1)),
                ),
                Instruction::new_double_operand_instruction(
                    Rc::new(Or),
                    vec![Rc::from(T0)],
                    Rc::from(A0),
                    Value::Reg(Rc::from(A1)),
                ),
            ],
        )]);
        assert_eq!(program.sections.len(), 1);
        // assert_eq!(program.errors.len(), 5);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_double_operand_reg_imm_instruction() -> Result<()> {
        use crate::assembler::tokens::Opcode::*;
        use crate::assembler::tokens::Register::*;
        let input = r"
.text
main:
    addi t0 <- t1, 8
    andi t1,t2,t3,s0,s1,s2,s3,s4,a0,a1,a2 <- a0, 200
    ori t0 <- t1, 8
    subi t0 <- t1, 8

.text
error:
    addi t0 <- t1, t2

.text
error2:
    addi <- t1, 7
        ";

        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![
                Instruction::new_double_operand_instruction(
                    Rc::new(Addi),
                    vec![Rc::from(T0)],
                    Rc::from(T1),
                    Value::Immediate(8),
                ),
                Instruction::new_double_operand_instruction(
                    Rc::new(Andi),
                    vec![
                        Rc::from(T1),
                        Rc::from(T2),
                        Rc::from(T3),
                        Rc::from(S0),
                        Rc::from(S1),
                        Rc::from(S2),
                        Rc::from(S3),
                        Rc::from(S4),
                        Rc::from(A0),
                        Rc::from(A1),
                        Rc::from(A2),
                    ],
                    Rc::from(A0),
                    Value::Immediate(200),
                ),
                Instruction::new_double_operand_instruction(
                    Rc::new(Ori),
                    vec![Rc::from(T0)],
                    Rc::from(T1),
                    Value::Immediate(8),
                ),
                Instruction::new_double_operand_instruction(
                    Rc::new(Subi),
                    vec![Rc::from(T0)],
                    Rc::from(T1),
                    Value::Immediate(8),
                ),
            ],
        )]);
        assert_eq!(program.sections.len(), 1);
        // assert_eq!(program.errors.len(), 5);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_single_operand_rr_instruction() -> Result<()> {
        use crate::assembler::sections::Value;
        use crate::assembler::tokens::Opcode::*;
        use crate::assembler::tokens::Register::*;
        let input = r"
.text
main:
    not t0, t2 <- t1
    sll t1, t2, t3, s0 <- t1
    sra t1 <- t1
    sla t1 <- t1
    mov t3, t2 <- t1
        ";

        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![
                Instruction::new_single_operand_instruction(
                    Rc::new(Not),
                    vec![Rc::from(T0), Rc::from(T2)],
                    Value::Reg(Rc::from(T1)),
                ),
                Instruction::new_single_operand_instruction(
                    Rc::new(Sll),
                    vec![Rc::from(T1), Rc::from(T2), Rc::from(T3), Rc::from(S0)],
                    Value::Reg(Rc::from(T1)),
                ),
                Instruction::new_single_operand_instruction(
                    Rc::new(Sra),
                    vec![Rc::from(T1)],
                    Value::Reg(Rc::from(T1)),
                ),
                Instruction::new_single_operand_instruction(
                    Rc::new(Sla),
                    vec![Rc::from(T1)],
                    Value::Reg(Rc::from(T1)),
                ),
                Instruction::new_single_operand_instruction(
                    Rc::new(Mov),
                    vec![Rc::from(T3), Rc::from(T2)],
                    Value::Reg(Rc::from(T1)),
                ),
            ],
        )]);
        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_single_operand_imm_instruction() -> Result<()> {
        use crate::assembler::sections::Value;
        use crate::assembler::tokens::Opcode::*;
        use crate::assembler::tokens::Register::*;
        let input = r"
.text
main:
    lui t0, t2, t1, ra <- 200
        ";

        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![Instruction::new_single_operand_instruction(
                Rc::new(Lui),
                vec![Rc::from(T0), Rc::from(T2), Rc::from(T1), Rc::from(Ra)],
                Value::Immediate(200),
            )],
        )]);
        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_branch_instruction() -> Result<()> {
        use crate::assembler::tokens::Register::*;
        let input = r"
.text
main:
    beq t0, t1, main
    bne t0, t1, kkk
    blt t0, t1, tubias
    bgt t0, t1, gepeto
        ";

        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![
                Instruction::new_branch_instruction(
                    BranchOp::Beq,
                    Rc::from(T0),
                    Rc::from(T1),
                    Rc::from("main"),
                ),
                Instruction::new_branch_instruction(
                    BranchOp::Bne,
                    Rc::from(T0),
                    Rc::from(T1),
                    Rc::from("kkk"),
                ),
                Instruction::new_branch_instruction(
                    BranchOp::Blt,
                    Rc::from(T0),
                    Rc::from(T1),
                    Rc::from("tubias"),
                ),
                Instruction::new_branch_instruction(
                    BranchOp::Bgt,
                    Rc::from(T0),
                    Rc::from(T1),
                    Rc::from("gepeto"),
                ),
            ],
        )]);
        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_no_operand_instruction() -> Result<()> {
        use crate::assembler::tokens::Opcode::*;
        let input = r"
.text
main:
    halt
    nop
        ";

        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![
                Instruction::new_no_operand_instruction(Rc::new(Halt)),
                Instruction::new_no_operand_instruction(Rc::new(Nop)),
            ],
        )]);
        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_jal() -> Result<()> {
        let input = r"
.text
main:
    jal tubias
        ";

        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![Instruction::new_jal_instruction(Rc::from("tubias"))],
        )]);
        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_write() -> Result<()> {
        let input = r"
.text
main:
    write 15 <- a1
    write label <- a2
";
        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![
                Instruction::new_write_instruction(
                    ImmediateOrLabel::Immediate(15),
                    Rc::new(Register::A1),
                ),
                Instruction::new_write_instruction(
                    ImmediateOrLabel::Label(Rc::from("label")),
                    Rc::new(Register::A2),
                ),
            ],
        )]);
        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_read() -> Result<()> {
        let input = r"
.text
main:
    read a1, a2, a3 <- 77
    read a1, a2, a3 <- label
";
        let program = create_program(input);

        let expected = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![
                Instruction::new_read_instruction(
                    ImmediateOrLabel::Immediate(77),
                    vec![
                        Rc::new(Register::A1),
                        Rc::new(Register::A2),
                        Rc::new(Register::A3),
                    ],
                ),
                Instruction::new_read_instruction(
                    ImmediateOrLabel::Label(Rc::from("label")),
                    vec![
                        Rc::new(Register::A1),
                        Rc::new(Register::A2),
                        Rc::new(Register::A3),
                    ],
                ),
            ],
        )]);
        assert_eq!(program.sections.len(), 1);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], expected);

        Ok(())
    }

    #[test]
    fn parse_text_and_data() -> Result<()> {
        let input = r"
.data
    test: .word 123
    test2: .byte 77

.text
main:
    read a1, a2, a3 <- 77
    read a1, a2, a3 <- label
";
        let program = create_program(input);

        let data = Sections::DataSection(vec![
            DataWrited {
                label: Rc::from("test"),
                kind: DataKind::Word(123),
            },
            DataWrited {
                label: Rc::from("test2"),
                kind: DataKind::Byte(77),
            },
        ]);

        let text = Sections::TextSection(vec![TextSegment::new_labeled_section(
            Rc::from("main"),
            vec![
                Instruction::new_read_instruction(
                    ImmediateOrLabel::Immediate(77),
                    vec![
                        Rc::new(Register::A1),
                        Rc::new(Register::A2),
                        Rc::new(Register::A3),
                    ],
                ),
                Instruction::new_read_instruction(
                    ImmediateOrLabel::Label(Rc::from("label")),
                    vec![
                        Rc::new(Register::A1),
                        Rc::new(Register::A2),
                        Rc::new(Register::A3),
                    ],
                ),
            ],
        )]);
        assert_eq!(program.sections.len(), 2);
        assert_eq!(program.errors.len(), 0);
        assert_eq!(program.sections[0], data);
        assert_eq!(program.sections[1], text);

        Ok(())
    }
}
