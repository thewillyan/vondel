use std::rc::Rc;

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
        instructions: Rc<[Instruction]>,
    },
    GlobalSection {
        labels: Vec<Rc<str>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Sections {
    TextSection(Vec<TextSegment>),
    DataSection(Vec<DataWrited>),
}

impl Sections {
    pub fn new_labeled_section(label: Rc<str>, instructions: Rc<[Instruction]>) -> TextSegment {
        TextSegment::LabeledSection {
            label,
            instructions,
        }
    }

    pub fn new_global_section(labels: Vec<Rc<str>>) -> TextSegment {
        TextSegment::GlobalSection { labels }
    }

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
