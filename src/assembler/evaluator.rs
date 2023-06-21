use anyhow::{bail, Result};
use std::rc::Rc;

use crate::{
    assembler::{
        sections::{Instruction, Sections, TextSegment, Value},
        tokens::{Opcode, Register},
    },
    uarch::mem::{CtrlStore, CtrlStoreBuilder},
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
            Instruction::SingleOperand(ins) => {
                Self::eval_single_op_inst(&ins.opcode, &ins.rd, &ins.rs1, state);
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

    fn eval_single_op_inst(
        op: &Opcode,
        rd: &Vec<Rc<Register>>,
        rs1: &Value,
        cs_state: &mut CsState,
    ) {
        let c_code = Self::get_c_code(rd);
        let mut mi = Microinstruction::new(cs_state.next_addr());
        mi.a = match rs1 {
            Value::Immediate(v) => {
                mi.immediate = *v;
                Microinstruction::IMM_A
            }
            Value::Reg(r) => Self::reg_a_code(r.as_ref()),
        };
        mi.c_bus = c_code;
        mi.b = Microinstruction::NO_B;

        match op {
            Opcode::Lui => mi.alu = 0b00011000,
            Opcode::Not => mi.alu = 0b00011010,
            Opcode::Sll => mi.alu = 0b10011000,
            Opcode::Sra => mi.alu = 0b01011000,
            Opcode::Sla => mi.alu = 0b11011000,
            Opcode::Mov => mi.alu = 0b00011000,
            _ => unreachable!("There is no other 'single operand' opcode"),
        }
        cs_state.add_instr(mi.get());
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
            Opcode::Add | Opcode::Addi => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b00111100;
                mi.a = Self::reg_a_code(rs1.as_ref());
                (mi.b, mi.immediate) = Self::val_b_code(rs2);
                cs_state.add_instr(mi.get());
            }
            Opcode::Sub | Opcode::Subi => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b00111111;
                mi.b = Self::reg_b_code(rs1.as_ref());
                (mi.a, mi.immediate) = Self::val_a_code(rs2);
                cs_state.add_instr(mi.get());
            }
            Opcode::Mul => {
                cs_state.add_complex_instr(|cs, addr| {
                    // Temp registers usage:
                    // - T0: gonna store the min(rs1, rs2)
                    // - T1: gonna store the max(rs1, rs2)
                    let mut next_addr = addr + 1;
                    let mut mi_list = Vec::new();

                    let branched_addr = next_addr | 0b100000000;
                    let mut branched_mi_list = Vec::new();

                    let mut mi = Microinstruction::new(next_addr);
                    // JUMP if rs1 > rs2
                    mi.jam = 0b010;
                    mi.alu = 0b00111111;
                    mi.a = Self::reg_a_code(rs1);
                    (mi.b, mi.immediate) = Self::val_b_code(rs2);
                    mi_list.push(mi.get());

                    // HAS NOT JUMPED, therefore rs1 <= rs2
                    // mv t0 <- rs1
                    next_addr += 1;
                    let mut mi = Microinstruction::new(next_addr);
                    mi.alu = 0b00011000;
                    mi.c_bus = Self::get_c_code(&vec![Rc::new(Register::T0)]);
                    mi.a = Self::reg_a_code(rs1);
                    mi_list.push(mi.get());
                    // mv t1, t2, rd <- rs2
                    next_addr += 1;
                    let mut mi = Microinstruction::new(next_addr);
                    mi.alu = 0b00011000;
                    mi.c_bus =
                        Self::get_c_code(&vec![Rc::new(Register::T1), Rc::new(Register::T2)])
                            | c_code;
                    mi.a = match rs2 {
                        Value::Reg(r) => Self::reg_a_code(r),
                        Value::Immediate(_) => unreachable!("Should't receive a immediate arg."),
                    };
                    let loop_addr = next_addr;
                    mi_list.push(mi.get());

                    // HAS JUMPED, therefore rs1 > rs2
                    // mv t0 <- rs2
                    let mut mi = Microinstruction::new(branched_addr + 1);
                    mi.alu = 0b00011000;
                    mi.c_bus = Self::get_c_code(&vec![Rc::new(Register::T0)]);
                    mi.a = match rs2 {
                        Value::Reg(r) => Self::reg_a_code(r),
                        Value::Immediate(_) => unreachable!("Should't receive a immediate arg."),
                    };
                    branched_mi_list.push(mi.get());
                    // mv t1, t2, rd <- rs1
                    let mut mi = Microinstruction::new(loop_addr);
                    mi.alu = 0b00011000;
                    mi.c_bus =
                        Self::get_c_code(&vec![Rc::new(Register::T1), Rc::new(Register::T2)])
                            | c_code;
                    mi.a = Self::reg_a_code(rs1);
                    branched_mi_list.push(mi.get());
                    cs.load_words(branched_addr, branched_mi_list);

                    // Intersection between the cases: t1 + .. + t1, t0-times
                    next_addr += 1;
                    // t0 <- t0 - 1 (special case because we subtract 1 without ussing immediate)
                    let mut mi = Microinstruction::new(next_addr);
                    mi.jam = 0b001;
                    mi.alu = 0b00110110;
                    mi.c_bus = Self::get_c_code(&vec![Rc::new(Register::T0)]);
                    mi.b = Self::reg_b_code(&Register::T0);
                    mi_list.push(mi.get());
                    // add t1, rd <- t1 + t2
                    let mut mi = Microinstruction::new(loop_addr);
                    mi.c_bus = Self::get_c_code(&vec![Rc::new(Register::T1)]) | c_code;
                    mi.alu = 0b00111100;
                    mi.a = Self::reg_a_code(&Register::T1);
                    mi.b = Self::reg_b_code(&Register::T2);
                    mi_list.push(mi.get());

                    cs.load_words(addr, mi_list);
                    next_addr | 0b100000000
                });
            }
            _ => unimplemented!(),
        }
    }

    /// Returns a pair of (A bus code, Immediate).
    fn val_a_code(val: &Value) -> (u8, u8) {
        match val {
            Value::Reg(r) => (Self::reg_a_code(r), 0),
            Value::Immediate(imm) => (Microinstruction::IMM_A, *imm),
        }
    }

    /// Returns a pair of (B bus code, Immediate).
    fn val_b_code(val: &Value) -> (u8, u8) {
        match val {
            Value::Reg(r) => (Self::reg_b_code(r), 0),
            Value::Immediate(imm) => (Microinstruction::IMM_B, *imm),
        }
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

    pub fn next_addr(&self) -> u16 {
        (self.curr_addr & 0b011111111) + 1
    }

    pub fn add_instr(&mut self, inst: u64) {
        let b = self.builder.set_word(self.curr_addr, inst);
        self.curr_addr = self.next_addr();
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
    pub mem: u8,
    pub a: u8,
    pub b: u8,
    pub immediate: u8,
}

impl Microinstruction {
    pub const HALT: u64 = u64::MAX;
    pub const IMM_A: u8 = 0b01000;
    pub const IMM_B: u8 = 0b00011;
    pub const NO_B: u8 = 0b11111;

    /// Creates a new microinstruction.
    pub fn new(next_addr: u16) -> Self {
        Self {
            next: next_addr,
            jam: 0,
            alu: 0,
            c_bus: 0,
            mem: 0,
            a: 0b11111,
            b: 0b11111,
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

        mi <<= 3;
        mi |= self.mem as u64;

        mi <<= 5;
        mi |= self.a as u64;

        mi <<= 5;
        mi |= self.b as u64;

        mi <<= 8;
        mi |= self.immediate as u64;
        mi
    }
}

#[allow(clippy::unusual_byte_groupings)]
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
    fn microinstruction() {
        let mut mi = Microinstruction::new(0);
        mi.next = 0b000000001;
        mi.jam = 0b000;
        mi.alu = 0b00111100;
        mi.c_bus = 0b00001100000000000000;
        mi.mem = 000;
        mi.a = 0b01011;
        mi.b = 0b01100;
        mi.immediate = 0b00000000;

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_00111100_00001100000000000000_000_01011_01100_00000000;

        assert_eq!(expected, mi.get());
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
        let expected = 0b000000001_000_00111100_00000000000000001100_000_01010_10011_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn basic_addi() {
        // add a0, a1 <- t0, 5
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                Rc::new(Opcode::Addi),
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
        let expected = 0b000000001_000_00111100_00000000000000001100_000_01010_00011_00000101;

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
        let expected = 0b000000001_000_00111111_00000000000000001100_000_11000_00101_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn basic_subi() {
        // add a0, a1 <- t0, 7
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                Rc::new(Opcode::Subi),
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
        let expected = 0b000000001_000_00111111_00000000000000001100_000_01000_00101_00000111;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_lui() {
        let instructions = vec![
            Instruction::new_single_operand_instruction(
                Rc::new(Opcode::Lui),
                vec![Rc::new(Register::A3)],
                Value::Immediate(1),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_00011000_00000000000000000001_000_01000_11111_00000001;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_mov() {
        let instructions = vec![
            Instruction::new_single_operand_instruction(
                Rc::new(Opcode::Mov),
                vec![Rc::new(Register::A3)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_00011000_00000000000000000001_000_00000_11111_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_not() {
        let instructions = vec![
            Instruction::new_single_operand_instruction(
                Rc::new(Opcode::Not),
                vec![Rc::new(Register::A3), Rc::new(Register::A2)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_00011010_00000000000000000011_000_00000_11111_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_sll() {
        let instructions = vec![
            Instruction::new_single_operand_instruction(
                Rc::new(Opcode::Sll),
                vec![Rc::new(Register::A3), Rc::new(Register::A2)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_10011000_00000000000000000011_000_00000_11111_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_sla() {
        let instructions = vec![
            Instruction::new_single_operand_instruction(
                Rc::new(Opcode::Sla),
                vec![Rc::new(Register::A3), Rc::new(Register::A2)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_11011000_00000000000000000011_000_00000_11111_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_sra() {
        let instructions = vec![
            Instruction::new_single_operand_instruction(
                Rc::new(Opcode::Sra),
                vec![Rc::new(Register::A3), Rc::new(Register::A2)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_01011000_00000000000000000011_000_00000_11111_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn mul() {
        // mul a0 <- a1, a2
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                Rc::new(Opcode::Mul),
                vec![Rc::new(Register::A0)],
                Rc::new(Register::A1),
                Value::Reg(Rc::new(Register::A2)),
            ),
            Instruction::new_no_operand_instruction(Rc::new(Opcode::Halt)),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::eval(&secs).firmware();

        let no_branch_mcode: Vec<u64> = vec![
            // JUMP if a1 > a2 (a2 - a1 < 0)
            0b000000001_010_00111111_00000000000000000000_000_10110_10010_00000000,
            // CASE 1 (has not branched)
            // Copy a1 into t0
            0b000000010_000_00011000_00000100000000000000_000_10110_11111_00000000,
            // Copy a2 into t1, t2, a0
            0b000000011_000_00011000_00000011000000001000_000_10111_11111_00000000,
            // Intersection between the cases: r13 + .. + r13, r14-times
            // t0 <- t0 - 1 or JUMP if (t0 - 1) = 0
            0b000000100_001_00110110_00000100000000000000_000_11111_00101_00000000,
            // a0, t1 <- t1 + t2
            0b000000011_000_00111100_00000010000000001000_000_01011_00111_00000000,
        ];

        for (addr, &mi) in no_branch_mcode.iter().enumerate() {
            // println!("addr:     {:09b}", addr);
            // eprintln!("expected: {:061b}", mi);
            // eprintln!("got:      {:061b}", firmware[addr]);
            assert_eq!(mi, firmware[addr]);
        }

        let brached_mcode: Vec<u64> = vec![
            // CASE 2: has branched
            // Copy a2 into t0
            0b100000010_000_00011000_00000100000000000000_000_10111_11111_00000000,
            // Copy a1 into t1, t2, a0
            0b000000011_000_00011000_00000011000000001000_000_10110_11111_00000000,
        ];

        for (addr, &mi) in brached_mcode.iter().enumerate() {
            let addr = addr + 0b100000001;
            // println!("addr:     {:09b}", addr);
            // eprintln!("expected: {:061b}", mi);
            // eprintln!("got:      {:061b}", firmware[addr]);
            assert_eq!(mi, firmware[addr]);
        }
    }
}
