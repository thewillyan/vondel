use std::rc::Rc;

use crate::assembler::tokens::Register;

#[derive(Debug, PartialEq)]
pub enum DoubleOperandOpcode {
    Add,
    Sub,
    Mul,
    Mul2,
    Muli,
    Div,
    Divi,
    Mod,
    Modi,
    And,
    Or,
    Xor,
    Xori,
    Addi,
    Andi,
    Ori,
    Subi,
}

#[derive(Debug, PartialEq)]
pub enum SingleOperandOpcode {
    Lui,
    Not,
    Sll,
    Sra,
    Sla,
    Mov,
}

#[derive(Debug, PartialEq)]
pub enum NoOperandOpcode {
    Halt,
    Nop,
}

#[derive(Debug, PartialEq)]
pub enum BranchOp {
    Beq,
    Bne,
    Blt,
    Bgt,
}
#[derive(Debug, PartialEq)]
pub enum Value {
    Immediate(u8),
    Reg(Rc<Register>),
    Label(Rc<str>),
}

#[derive(Debug, PartialEq)]
pub enum ImmediateOrLabel {
    Immediate(u8),
    Label(Rc<str>),
}

#[derive(Debug, PartialEq)]
pub struct DoubleOperandInstruction {
    pub opcode: DoubleOperandOpcode,
    pub rd: Vec<Rc<Register>>,
    pub rs1: Rc<Register>,
    pub rs2: Value,
}

#[derive(Debug, PartialEq)]
pub struct SingleOperandInstruction {
    pub opcode: SingleOperandOpcode,
    pub rd: Vec<Rc<Register>>,
    pub rs1: Value,
}

#[derive(Debug, PartialEq)]
pub struct BranchInstruction {
    pub opcode: BranchOp,
    pub rs1: Rc<Register>,
    pub rs2: Rc<Register>,
    pub label: Rc<str>,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    DoubleOperand(DoubleOperandInstruction),
    SingleOperand(SingleOperandInstruction),
    Branch(BranchInstruction),
    NoOperand(NoOperandOpcode),
    WriteInstruction(ImmediateOrLabel, Rc<Register>),
    ReadInstruction(ImmediateOrLabel, Vec<Rc<Register>>),
    Jal(Rc<str>),
}

impl Instruction {
    pub fn new_double_operand_instruction(
        opcode: DoubleOperandOpcode,
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
        opcode: SingleOperandOpcode,
        rd: Vec<Rc<Register>>,
        rs1: Value,
    ) -> Instruction {
        Instruction::SingleOperand(SingleOperandInstruction { opcode, rd, rs1 })
    }

    pub fn new_no_operand_instruction(opcode: NoOperandOpcode) -> Instruction {
        Instruction::NoOperand(opcode)
    }

    pub fn new_jal_instruction(label: Rc<str>) -> Instruction {
        Instruction::Jal(label)
    }

    pub fn new_write_instruction(
        immediate_or_label: ImmediateOrLabel,
        rd: Rc<Register>,
    ) -> Instruction {
        Instruction::WriteInstruction(immediate_or_label, rd)
    }

    pub fn new_read_instruction(
        immediate_or_label: ImmediateOrLabel,
        rd: Vec<Rc<Register>>,
    ) -> Instruction {
        Instruction::ReadInstruction(immediate_or_label, rd)
    }

    pub fn new_branch_instruction(
        opcode: BranchOp,
        rs1: Rc<Register>,
        rs2: Rc<Register>,
        label: Rc<str>,
    ) -> Instruction {
        Instruction::Branch(BranchInstruction {
            opcode,
            rs1,
            rs2,
            label,
        })
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
