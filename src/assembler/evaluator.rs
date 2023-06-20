use crate::assembler::{
    sections::{Instruction, Sections, TextSegment},
    tokens::Opcode,
};

pub struct AsmEvaluator {}

impl AsmEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval(secs: &Sections) {
        match secs {
            Sections::TextSection(txt_segs) => {
                for seg in txt_segs {
                    Self::eval_txt_seg(seg);
                }
            }
            Sections::DataSection(data) => unimplemented!(),
        }
    }

    pub fn eval_txt_seg(txt_seg: &TextSegment) {
        match txt_seg {
            // ignoring labels for now
            TextSegment::LabeledSection {
                label: _,
                instructions,
            } => {
                for inst in instructions {
                    Self::eval_inst(inst);
                }
            }
            TextSegment::GlobalSection { label: _ } => unimplemented!(),
        }
    }

    pub fn eval_inst(inst: &Instruction) {
        let mi = 0u64;
        match inst {
            Instruction::DoubleOperand(inst) => match *inst.opcode {
                Opcode::Add => (),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
}
