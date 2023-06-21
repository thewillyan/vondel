use std::rc::Rc;

use crate::uarch::mem::CtrlStoreBuilder;
use crate::{
    assembler::{
        sections::{Instruction, Sections, TextSegment, Value},
        tokens::{Opcode, Register},
    },
    uarch::mem::CtrlStore,
};

pub struct AsmEvaluator {}

impl AsmEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval(secs: &Sections) -> CtrlStore {
        let mut cs_state = CsState::new();

        match secs {
            Sections::TextSection(txt_segs) => {
                for seg in txt_segs {
                    Self::eval_txt_seg(seg, &mut cs_state);
                }
            }
            Sections::DataSection(_data) => unimplemented!(),
        }

        cs_state.build_cs()
    }

    pub fn eval_txt_seg(txt_seg: &TextSegment, state: &mut CsState) {
        match txt_seg {
            // ignoring labels for now
            TextSegment::LabeledSection {
                label: _,
                instructions,
            } => {
                for inst in instructions {
                    Self::eval_inst(inst, state);
                }
            }
            TextSegment::GlobalSection { label: _ } => unimplemented!(),
        }
    }

    pub fn eval_inst(inst: &Instruction, state: &mut CsState) {
        match inst {
            Instruction::DoubleOperand(inst) => {
                Self::eval_double_op_inst(&inst.opcode, &inst.rd, &inst.rs1, &inst.rs2, state);
            }
            Instruction::NoOperand(opcode) => Self::eval_no_op_inst(opcode.as_ref(), state),
            _ => unimplemented!(),
        }
    }

    fn eval_no_op_inst(opcode: &Opcode, state: &mut CsState) {
        match opcode {
            Opcode::Halt => state.add_instr(Microinstruction::HALT),
            _ => unreachable!("There is no other 'no operand' opcode"),
        }
    }

    fn eval_double_op_inst(
        opcode: &Opcode,
        rd: &Vec<Rc<Register>>,
        rs1: &Rc<Register>,
        rs2: &Value,
        cs_state: &mut CsState,
    ) {
        let c_code = Self::get_c_code(rd);
        match opcode {
            Opcode::Add => {
                let mut mi = Microinstruction::new(cs_state.addr() + 1);
                mi.c_bus = c_code;
                mi.alu = 0b00111100;
                mi.a = Self::reg_a_code(rs1.as_ref());
                mi.b = match rs2 {
                    Value::Reg(r) => Self::reg_b_code(r),
                    Value::Immediate(imm) => {
                        mi.immediate = *imm;
                        Microinstruction::IMM_B
                    }
                };
                cs_state.add_instr(mi.get());
            }
            Opcode::Sub => {
                let mut mi = Microinstruction::new(cs_state.addr() + 1);
                mi.c_bus = c_code;
                mi.alu = 0b00111111;
                mi.b = Self::reg_b_code(rs1.as_ref());
                mi.a = match rs2 {
                    Value::Reg(r) => Self::reg_a_code(r),
                    Value::Immediate(imm) => {
                        mi.immediate = *imm;
                        Microinstruction::IMM_A
                    }
                };
                cs_state.add_instr(mi.get());
            }
            _ => unimplemented!(),
        }
    }

    /// Returns the c bus field content of the MI. (20 bits)
    fn get_c_code(regs: &Vec<Rc<Register>>) -> u32 {
        let mut c_code = 0;
        for reg in regs {
            match reg.as_ref() {
                Register::Mdr => c_code |= 1 << 19,
                Register::Mar => c_code |= 1 << 18,
                Register::Pc => c_code |= 1 << 17,
                Register::Lv => c_code |= 1 << 16,
                Register::Ra => c_code |= 1 << 15,
                Register::T0 => c_code |= 1 << 14,
                Register::T1 => c_code |= 1 << 13,
                Register::T2 => c_code |= 1 << 12,
                Register::T3 => c_code |= 1 << 11,
                Register::S0 => c_code |= 1 << 10,
                Register::S1 => c_code |= 1 << 9,
                Register::S2 => c_code |= 1 << 8,
                Register::S3 => c_code |= 1 << 7,
                Register::S4 => c_code |= 1 << 6,
                Register::S5 => c_code |= 1 << 5,
                Register::S6 => c_code |= 1 << 4,
                Register::A0 => c_code |= 1 << 3,
                Register::A1 => c_code |= 1 << 2,
                Register::A2 => c_code |= 1 << 1,
                Register::A3 => c_code |= 1,
                _ => unreachable!("Cannot write on other registers!"),
            }
        }
        c_code
    }

    fn reg_a_code(reg: &Register) -> u8 {
        match reg {
            Register::Mdr => 0,
            Register::Pc => 1,
            Register::Mbr => 2,
            Register::Mbru => 3,
            Register::Mbr2 => 4,
            Register::Mbr2u => 5,
            Register::Lv => 6,
            Register::Cpp => 7,
            Register::Ra => 9,
            Register::T0 => 10,
            Register::T1 => 11,
            Register::T2 => 12,
            Register::T3 => 13,
            Register::S0 => 14,
            Register::S1 => 15,
            Register::S2 => 16,
            Register::S3 => 17,
            Register::S4 => 18,
            Register::S5 => 19,
            Register::S6 => 20,
            Register::A0 => 21,
            Register::A1 => 22,
            Register::A2 => 23,
            Register::A3 => 24,
            _ => unreachable!("Cannot write other registers to the A bus!"),
        }
    }

    pub fn reg_b_code(reg: &Register) -> u8 {
        match reg {
            Register::Mdr => 0,
            Register::Lv => 1,
            Register::Cpp => 2,
            Register::Ra => 4,
            Register::T0 => 5,
            Register::T1 => 6,
            Register::T2 => 7,
            Register::T3 => 8,
            Register::S0 => 9,
            Register::S1 => 10,
            Register::S2 => 11,
            Register::S3 => 12,
            Register::S4 => 13,
            Register::S5 => 14,
            Register::S6 => 15,
            Register::A0 => 16,
            Register::A1 => 17,
            Register::A2 => 18,
            Register::A3 => 19,
            _ => unreachable!("Cannot write other registers to the B bus!"),
        }
    }
}

#[derive(Default)]
pub struct CsState {
    builder: CtrlStoreBuilder,
    curr_addr: u16,
}

impl CsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn addr(&self) -> u16 {
        self.curr_addr
    }

    pub fn add_instr(&mut self, inst: u64) {
        let b = self.builder.set_word(self.curr_addr, inst);
        self.curr_addr += 1;
    }

    /// Add a complex instructions from a functions that returns the address which
    /// the next instructions should be stored.
    pub fn add_complex_instr<F>(&mut self, func: F)
    where
        F: FnOnce(&mut CtrlStoreBuilder, u16) -> u16,
    {
        self.curr_addr = (func)(&mut self.builder, self.curr_addr)
    }

    pub fn build_cs(self) -> CtrlStore {
        self.builder.build()
    }
}

struct Microinstruction {
    pub next: u16,
    pub jam: u8,
    pub alu: u8,
    pub c_bus: u32,
    pub a: u8,
    pub b: u8,
    pub immediate: u8,
}

impl Microinstruction {
    pub const HALT: u64 = u64::MAX;
    pub const IMM_A: u8 = 0b01000;
    pub const IMM_B: u8 = 0b00011;

    /// Creates a new microinstruction.
    pub fn new(next_addr: u16) -> Self {
        Self {
            next: next_addr,
            jam: 0,
            alu: 0,
            c_bus: 0,
            a: u8::MAX,
            b: u8::MAX,
            immediate: 0,
        }
    }

    /// Get value of the Microinstruction
    pub fn get(&self) -> u64 {
        let mut mi = self.next as u64;
        mi <<= 3;
        mi |= self.jam as u64;

        mi <<= 8;
        mi |= self.alu as u64;

        mi <<= 20;
        mi |= self.c_bus as u64;

        mi <<= 5;
        mi |= self.a as u64;

        mi <<= 5;
        mi |= self.b as u64;

        mi <<= 8;
        mi |= self.immediate as u64;
        mi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cbus_eval() {
        let regs = vec![
            Rc::new(Register::Mdr),
            Rc::new(Register::A3),
            Rc::new(Register::S6),
            Rc::new(Register::S5),
            Rc::new(Register::Ra),
        ];
        assert_eq!(0b10001000000000110001, AsmEvaluator::get_c_code(&regs));
    }

    #[test]
    fn basic_add() {
        // add a0, a1 <- t0, a3
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                Rc::new(Opcode::Add),
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Reg(Rc::new(Register::A3)),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_00111100_00000000000000001100_01010_10011_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn add_with_immediate() {
        // add a0, a1 <- t0, 5
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                Rc::new(Opcode::Add),
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Immediate(5),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_00111100_00000000000000001100_01010_00011_00000101;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn basic_sub() {
        // sub a0, a1 <- t0, a3
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                Rc::new(Opcode::Sub),
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Reg(Rc::new(Register::A3)),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_00111111_00000000000000001100_11000_00101_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn sub_with_immediate() {
        // add a0, a1 <- t0, 7
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                Rc::new(Opcode::Sub),
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Immediate(7),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_00111111_00000000000000001100_01000_00101_00000111;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }
}
