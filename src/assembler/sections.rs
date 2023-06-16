use crate::assembler::tokens::{Opcode, Register};

#[derive(Debug, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub rd: Register,
    pub rs1: Register,
    pub rs2: Register,
}

#[derive(Debug, PartialEq)]
pub enum DataKind {
    Byte(u8),
    Word(u32),
}

#[derive(Debug, PartialEq)]
pub struct DataWrited {
    pub kind: DataKind,
    pub label: String,
}

#[derive(Debug, PartialEq)]
pub enum Sections {
    LabeledSection {
        label: String,
        instructions: Vec<Instruction>,
    },
    DataSection(Vec<DataWrited>),
}

impl Sections {
    pub fn new_labeled_section(label: String, instructions: Vec<Instruction>) -> Self {
        Sections::LabeledSection {
            label,
            instructions,
        }
    }

    pub fn new_data_section(data: Vec<DataWrited>) -> Self {
        Sections::DataSection(data)
    }

    pub fn new_data_writed(kind: DataKind, label: String) -> DataWrited {
        DataWrited { kind, label }
    }
}
