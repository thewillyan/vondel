use std::rc::Rc;

use crate::assembler::tokens::{Opcode, Register};

#[derive(Debug, PartialEq)]
pub enum Value {
    Immediate(u8),
    Reg(Rc<Register>),
}

#[derive(Debug, PartialEq)]
pub struct DoubleOperandInstruction {
    pub opcode: Rc<Opcode>,
    pub rd: Vec<Rc<Register>>,
    pub rs1: Rc<Register>,
    pub rs2: Value,
}

#[derive(Debug, PartialEq)]
pub struct SingleOperandInstruction {
    pub opcode: Rc<Opcode>,
    pub rd: Vec<Rc<Register>>,
    pub rs1: Rc<Register>,
}

#[derive(Debug, PartialEq)]
pub struct BranchInstruction {
    pub opcode: Rc<Opcode>,
    pub rs1: Rc<Register>,
    pub rs2: Rc<Register>,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    DoubleOperand(DoubleOperandInstruction),
    SingleOperand(SingleOperandInstruction),
    Branch(BranchInstruction),
    NoOperand(Rc<Opcode>),
}

impl Instruction {
    pub fn new_double_operand_instruction(
        opcode: Rc<Opcode>,
        rd: Vec<Rc<Register>>,
        rs1: Rc<Register>,
        rs2: Value,
    ) -> Instruction {
        Instruction::DoubleOperand(DoubleOperandInstruction {
            opcode,
            rd,
            rs1,
            rs2,
        })
    }

    pub fn new_single_operand_instruction(
        opcode: Rc<Opcode>,
        rd: Vec<Rc<Register>>,
        rs1: Rc<Register>,
    ) -> Instruction {
        Instruction::SingleOperand(SingleOperandInstruction { opcode, rd, rs1 })
    }

    pub fn new_no_operand_instruction(opcode: Rc<Opcode>) -> Instruction {
        Instruction::NoOperand(opcode)
    }

    pub fn new_branch_instruction(
        opcode: Rc<Opcode>,
        rs1: Rc<Register>,
        rs2: Rc<Register>,
    ) -> Instruction {
        Instruction::Branch(BranchInstruction { opcode, rs1, rs2 })
    }
}

#[derive(Debug, PartialEq)]
pub enum DataKind {
    Byte(u8),
    Word(i32),
}

#[derive(Debug, PartialEq)]
pub struct DataWrited {
    pub kind: DataKind,
    pub label: Rc<str>,
}

#[derive(Debug, PartialEq)]
pub enum TextSegment {
    LabeledSection {
        label: Rc<str>,
        instructions: Vec<Instruction>,
    },
    GlobalSection {
        label: Rc<str>,
    },
}

impl TextSegment {
    pub fn new_labeled_section(label: Rc<str>, instructions: Vec<Instruction>) -> TextSegment {
        TextSegment::LabeledSection {
            label,
            instructions,
        }
    }

    pub fn new_global_section(label: Rc<str>) -> TextSegment {
        TextSegment::GlobalSection { label }
    }
}

#[derive(Debug, PartialEq)]
pub enum Sections {
    TextSection(Vec<TextSegment>),
    DataSection(Vec<DataWrited>),
}

impl Sections {
    pub fn new_data_section(data: Vec<DataWrited>) -> Self {
        Sections::DataSection(data)
    }

    pub fn new_text_section(text: Vec<TextSegment>) -> Self {
        Sections::TextSection(text)
    }

    pub fn new_data_writed(kind: DataKind, label: Rc<str>) -> DataWrited {
        DataWrited { kind, label }
    }
}
