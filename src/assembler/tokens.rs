use super::parser::ParserError;
use anyhow::{bail, Result};
use clap::Parser;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub struct TokWithCtx {
    pub tok: Rc<AsmToken>,
    pub cur_line: usize,
    pub cur_column: usize,
}

impl TokWithCtx {
    pub fn new(tok: AsmToken, cur_line: usize, cur_column: usize) -> Self {
        TokWithCtx {
            tok: Rc::new(tok),
            cur_line,
            cur_column,
        }
    }
}

//RISC-V ABI
#[derive(Debug, PartialEq, Clone)]
pub enum Register {
    Ra,
    Sp,
    Cpp,
    Lv,
    T0,
    T1,
    T2,
    T3,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    A0,
    A1,
    A2,
    A3,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Opcode {
    // Integer Register-Immediate Instructions
    // Addi,
    // Slti,
    // Andi,
    // Ori,
    // Xori,
    // Slli,
    // Srli,
    // Lui,
    // Auipc,
    // Integer Register-Register Operations
    Add,
    Sub,
    Slt,
    And,
    Or,
    // Shift left 1 bit
    Sll,
    Srl,
    Nop,
    // Unconditional Jumps
    Jal,
    // Conditional Branches
    Beq,
    Bne,
    Blt,
    Bge,
    // Multiplication Operations
    Mul,
    // Read and Write
    // x ,addr ,rd
    Read,
    Write,
    // Halt
    Halt,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PseudoInstruction {
    Mv,
    Neg,
    // Set Operations
    Seqz,
    Snez,
    Sltz,
    Sgtz,
    // Branch Zero
    Beqz,
    Bnez,
    Blez,
    Bgez,
    Bltz,
    Bgtz,
    // Branch If
    Bgt,
    Ble,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PseudoOps {
    Data,
    Word,
    Byte,
    Text,
    Global,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AsmToken {
    Number(Rc<str>),
    Label(Rc<str>),
    Reg(Register),
    Opcode(Opcode),
    PseudoIns(PseudoInstruction),
    PseudoOp(PseudoOps),
    Comma,
    Colon,
    Illegal,
    Eof,
    Assign,
}

impl AsmToken {
    pub fn get_pseudo_op(&self) -> Result<PseudoOps> {
        match self {
            AsmToken::PseudoOp(op) => Ok(op.clone()),
            _ => bail!("Expected PseudoOp, got {:?}", self),
        }
    }

    pub fn get_label(&self, cur_line: usize, cur_column: usize) -> Result<Rc<str>> {
        match self {
            AsmToken::Label(label) => Ok(Rc::clone(label)),
            _ => {
                bail!(ParserError::ExpectedToken {
                    expected: format!("{:?}", AsmToken::Label(Rc::from(""))),
                    found: format!("{:?}", self),
                    cur_line,
                    cur_column
                })
            }
        }
    }

    pub fn get_number(&self, cur_line: usize, cur_column: usize) -> Result<Rc<str>> {
        match self {
            AsmToken::Number(number) => Ok(Rc::clone(number)),
            _ => {
                bail!(ParserError::ExpectedToken {
                    expected: format!("{:?}", AsmToken::Number(Rc::from(""))),
                    found: format!("{:?}", self),
                    cur_line,
                    cur_column
                })
            }
        }
    }

    pub fn name_to_tok(input: &str) -> AsmToken {
        match input {
            //PseudoOps
            pseudo if pseudo.starts_with('.') => match pseudo {
                ".data" => AsmToken::PseudoOp(PseudoOps::Data),
                ".word" => AsmToken::PseudoOp(PseudoOps::Word),
                ".byte" => AsmToken::PseudoOp(PseudoOps::Byte),
                ".text" => AsmToken::PseudoOp(PseudoOps::Text),
                ".global" => AsmToken::PseudoOp(PseudoOps::Global),
                _ => AsmToken::Illegal,
            },

            //OPCODES
            // "addi" => AsmToken::Opcode(Opcode::Addi),
            // "slti" => AsmToken::Opcode(Opcode::Slti),
            // "andi" => AsmToken::Opcode(Opcode::Andi),
            // "ori" => AsmToken::Opcode(Opcode::Ori),
            // "xori" => AsmToken::Opcode(Opcode::Xori),
            // "slli" => AsmToken::Opcode(Opcode::Slli),
            // "srli" => AsmToken::Opcode(Opcode::Srli),
            // "lui" => AsmToken::Opcode(Opcode::Lui),
            // "auipc" => AsmToken::Opcode(Opcode::Auipc),
            "add" => AsmToken::Opcode(Opcode::Add),
            "sub" => AsmToken::Opcode(Opcode::Sub),
            "slt" => AsmToken::Opcode(Opcode::Slt),
            "and" => AsmToken::Opcode(Opcode::And),
            "or" => AsmToken::Opcode(Opcode::Or),
            "sll" => AsmToken::Opcode(Opcode::Sll),
            "srl" => AsmToken::Opcode(Opcode::Srl),
            "nop" => AsmToken::Opcode(Opcode::Nop),
            "jal" => AsmToken::Opcode(Opcode::Jal),
            "beq" => AsmToken::Opcode(Opcode::Beq),
            "bne" => AsmToken::Opcode(Opcode::Bne),
            "blt" => AsmToken::Opcode(Opcode::Blt),
            "bge" => AsmToken::Opcode(Opcode::Bge),
            "mul" => AsmToken::Opcode(Opcode::Mul),
            "halt" => AsmToken::Opcode(Opcode::Halt),
            "read" => AsmToken::Opcode(Opcode::Read),
            "write" => AsmToken::Opcode(Opcode::Write),

            //PSEUDO INSTRUCTIONS
            "mv" => AsmToken::PseudoIns(PseudoInstruction::Mv),
            "neg" => AsmToken::PseudoIns(PseudoInstruction::Neg),
            "seqz" => AsmToken::PseudoIns(PseudoInstruction::Seqz),
            "snez" => AsmToken::PseudoIns(PseudoInstruction::Snez),
            "sltz" => AsmToken::PseudoIns(PseudoInstruction::Sltz),
            "sgtz" => AsmToken::PseudoIns(PseudoInstruction::Sgtz),
            "beqz" => AsmToken::PseudoIns(PseudoInstruction::Beqz),
            "bnez" => AsmToken::PseudoIns(PseudoInstruction::Bnez),
            "blez" => AsmToken::PseudoIns(PseudoInstruction::Blez),
            "bgez" => AsmToken::PseudoIns(PseudoInstruction::Bgez),
            "bltz" => AsmToken::PseudoIns(PseudoInstruction::Bltz),
            "bgtz" => AsmToken::PseudoIns(PseudoInstruction::Bgtz),
            "bgt" => AsmToken::PseudoIns(PseudoInstruction::Bgt),
            "ble" => AsmToken::PseudoIns(PseudoInstruction::Ble),

            //REGISTERS
            "ra" | "x0" => AsmToken::Reg(Register::Ra),
            "sp" | "x1" => AsmToken::Reg(Register::Sp),
            "cpp" | "x2" => AsmToken::Reg(Register::Cpp),
            "lv" | "x3" => AsmToken::Reg(Register::Lv),
            "t0" | "x4" => AsmToken::Reg(Register::T0),
            "t1" | "x5" => AsmToken::Reg(Register::T1),
            "t2" | "x6" => AsmToken::Reg(Register::T2),
            "t3" | "x7" => AsmToken::Reg(Register::T3),
            "s0" | "x8" => AsmToken::Reg(Register::S0),
            "s1" | "x9" => AsmToken::Reg(Register::S1),
            "s2" | "x10" => AsmToken::Reg(Register::S2),
            "s3" | "x11" => AsmToken::Reg(Register::S3),
            "s4" | "x12" => AsmToken::Reg(Register::S4),
            "s5" | "x13" => AsmToken::Reg(Register::S5),
            "s6" | "x14" => AsmToken::Reg(Register::S6),
            "a0" | "x15" => AsmToken::Reg(Register::A0),
            "a1" | "x16" => AsmToken::Reg(Register::A1),
            "a2" | "x17" => AsmToken::Reg(Register::A2),
            "a3" | "x18" => AsmToken::Reg(Register::A3),

            _ => AsmToken::Label(Rc::from(input)),
        }
    }
}
