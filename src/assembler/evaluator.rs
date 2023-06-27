use anyhow::{bail, Result};
use std::{collections::HashMap, rc::Rc};

use crate::{
    assembler::{
        lexer::Lexer,
        parser::{Parser, Program},
        sections::{
            BranchOp, DoubleOperandOpcode, ImmediateOrLabel, Instruction, NoOperandOpcode,
            Sections, SingleOperandOpcode, TextSegment, Value,
        },
        tokens::Register,
    },
    uarch::mem::{CtrlStore, CtrlStoreBuilder},
};

use super::sections::{BranchInstruction, DataKind, DataWrited};

#[derive(Default)]
pub struct AsmEvaluator {
    values: HashMap<Rc<str>, u8>,
    addr: HashMap<Rc<str>, u8>,
    ram: Vec<u32>,
    unreachable: Vec<(Rc<str>, u16, Microinstruction)>,
}

impl AsmEvaluator {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            addr: HashMap::new(),
            ram: Vec::new(),
            unreachable: Vec::new(),
        }
    }

    pub fn evaluate_buffer(&mut self, buf: &str) -> Result<(CtrlStore, &[u32])> {
        let toks = Lexer::new(buf).get_deez_toks_w_ctx();
        let program = Parser::new(toks.into()).get_deez_program();

        self.eval_program(program)
    }

    pub fn eval_program(&mut self, prog: Program) -> Result<(CtrlStore, &[u32])> {
        if !prog.errors.is_empty() {
            eprintln!("Errors found while parsing the program.");
            for err in prog.errors {
                eprintln!("{}", err);
            }
            bail!("Errors found while parsing the program.");
        }

        let mut data = Vec::new();
        let mut text = Vec::new();

        for sec in prog.sections {
            match sec {
                Sections::TextSection(t) => {
                    text.extend(t);
                }
                Sections::DataSection(d) => data.extend(d),
            }
        }

        let mut cs = CsState::new();

        data.iter().for_each(|d| self.eval_data_seg(d));
        text.iter().for_each(|t| self.eval_txt_seg(t, &mut cs));
        self.resolve_unreachable(&mut cs);

        Ok((cs.build_cs(), &self.ram))
    }

    pub fn eval(&mut self, secs: &Sections) -> CtrlStore {
        let mut cs_state = CsState::new();

        match secs {
            Sections::TextSection(txt_segs) => {
                for seg in txt_segs {
                    self.eval_txt_seg(seg, &mut cs_state);
                }
            }
            Sections::DataSection(data) => {
                for seg in data {
                    self.eval_data_seg(seg);
                }
            }
        }

        cs_state.build_cs()
    }

    fn eval_data_seg(&mut self, data: &DataWrited) {
        let label = Rc::clone(&data.label);
        match data.kind {
            DataKind::Byte(b) => {
                self.values.insert(label, b);
            }
            DataKind::Word(w) => {
                self.values.insert(label, self.ram.len() as u8);
                self.ram.push(w as u32);
            }
        }
    }

    fn eval_txt_seg(&mut self, txt_seg: &TextSegment, state: &mut CsState) {
        match txt_seg {
            // ignoring labels for now
            TextSegment::LabeledSection {
                label,
                instructions,
            } => {
                self.addr.insert(Rc::clone(label), state.curr_addr as u8);
                for inst in instructions {
                    self.eval_inst(inst, state);
                }
            }
            TextSegment::GlobalSection { label: _ } => unimplemented!(),
        }
    }

    fn resolve_unreachable(&mut self, state: &mut CsState) {
        for (label, cs_addr, mut micro) in self.unreachable.drain(..) {
            let addr = *self
                .addr
                .get(label.as_ref())
                .expect("Should be defined before");

            micro.next = addr as u16;
            state.set_instr(cs_addr, micro.get());
        }
    }

    fn eval_inst(&mut self, inst: &Instruction, state: &mut CsState) {
        match inst {
            Instruction::DoubleOperand(inst) => {
                self.eval_double_op_inst(&inst.opcode, &inst.rd, &inst.rs1, &inst.rs2, state);
            }
            Instruction::SingleOperand(ins) => {
                self.eval_single_op_inst(&ins.opcode, &ins.rd, &ins.rs1, state);
            }
            Instruction::NoOperand(opcode) => self.eval_no_op_inst(opcode, state),
            Instruction::Branch(ins) => {
                self.eval_branch_inst(ins, state);
            }
            Instruction::Jal(label) => {
                self.eval_jal_inst(label, state);
            }
            Instruction::WriteInstruction(addr, rd) => {
                self.eval_write_inst(addr, rd, state);
            }
            Instruction::ReadInstruction(addr, rd) => {
                self.eval_read_inst(addr, rd, state);
            }
        }
    }

    fn eval_read_inst(
        &mut self,
        addr: &ImmediateOrLabel,
        rds: &Vec<Rc<Register>>,
        state: &mut CsState,
    ) {
        let mut read = Microinstruction::new(state.next_addr());
        read.c_bus = self.get_c_code(&vec![Rc::new(Register::Mar)]);
        read.alu = 0b000011000;
        read.mem = 0b010;
        read.a = Microinstruction::IMM_A;
        read.immediate = match addr {
            ImmediateOrLabel::Immediate(imm) => *imm,
            ImmediateOrLabel::Label(label) => *self
                .values
                .get(label.as_ref())
                .expect("Should be defined before"),
        };
        state.add_instr(read.get());

        let mut w_reg = Microinstruction::new(state.next_addr());
        w_reg.c_bus = self.get_c_code(rds);
        w_reg.alu = 0b000011000;
        w_reg.a = self.reg_a_code(&Register::Mdr);
        state.add_instr(w_reg.get());
    }

    fn eval_write_inst(&mut self, addr: &ImmediateOrLabel, rd: &Rc<Register>, state: &mut CsState) {
        let mut mdr = Microinstruction::new(state.next_addr());
        mdr.c_bus = self.get_c_code(&vec![Rc::new(Register::Mdr)]);
        mdr.alu = 0b000011000;
        mdr.a = self.reg_a_code(rd);
        state.add_instr(mdr.get());

        let mut mar = Microinstruction::new(state.next_addr());
        mar.c_bus = self.get_c_code(&vec![Rc::new(Register::Mar)]);
        mar.alu = 0b000011000;
        mar.mem = 0b100;
        mar.a = Microinstruction::IMM_A;
        mar.immediate = match addr {
            ImmediateOrLabel::Immediate(imm) => *imm,
            ImmediateOrLabel::Label(label) => *self
                .values
                .get(label.as_ref())
                .expect("Should be defined before"),
        };
        state.add_instr(mar.get());
    }

    fn eval_jal_inst(&mut self, label: &Rc<str>, state: &mut CsState) {
        let mut mi = Microinstruction::new(state.next_addr());
        match self.addr.get(label) {
            Some(v) => {
                mi.next = *v as u16;
            }
            None => {
                self.unreachable
                    .push((Rc::clone(label), state.curr_addr, mi.clone()));
            }
        }
        state.add_instr(mi.get());
    }

    fn eval_branch_inst(&mut self, ins: &BranchInstruction, state: &mut CsState) {
        let label = Rc::clone(&ins.label);

        let mut first = Microinstruction::new(state.next_addr());
        let mut second = None;
        let branched_addr = state.next_addr() | 0b100000000;
        first.a = self.reg_a_code(&ins.rs1);
        first.b = self.reg_b_code(&ins.rs2);
        first.alu = 0b000111111;

        let mut branched = Microinstruction::new(branched_addr);
        let mut second_branched = None;

        match self.addr.get(&label) {
            Some(v) => {
                branched.next = *v as u16;
            }
            None => {
                self.unreachable
                    .push((label, branched_addr, branched.clone()));
            }
        }

        match ins.opcode {
            BranchOp::Beq => first.jam = 0b001,
            BranchOp::Bne => {
                first.jam = 0b010;
                let mut mi = first.clone();
                mi.next += 1;
                mi.a = self.reg_a_code(&ins.rs2);
                mi.b = self.reg_b_code(&ins.rs1);
                second = Some(mi);
                second_branched = Some(branched.clone());

                let label = Rc::clone(&ins.label);
                if self.addr.get(&label).is_none() {
                    self.unreachable
                        .push((label, branched_addr + 1, branched.clone()));
                }
            }
            BranchOp::Blt => {
                first.jam = 0b010;
                first.a = self.reg_a_code(&ins.rs2);
                first.b = self.reg_b_code(&ins.rs1);
            }
            BranchOp::Bgt => first.jam = 0b010,
        }

        state.add_instr(first.get());
        if let Some(mi) = second {
            state.add_instr(mi.get());
        }

        state.set_instr(branched_addr, branched.get());
        if let Some(mi) = second_branched {
            state.set_instr(branched_addr + 1, mi.get());
        }
    }

    fn eval_no_op_inst(&mut self, opcode: &NoOperandOpcode, state: &mut CsState) {
        match opcode {
            NoOperandOpcode::Halt => state.add_instr(Microinstruction::HALT),
            NoOperandOpcode::Nop => state.add_instr(Microinstruction::new(state.next_addr()).get()),
        }
    }

    fn eval_single_op_inst(
        &mut self,
        op: &SingleOperandOpcode,
        rd: &Vec<Rc<Register>>,
        rs1: &Value,
        cs_state: &mut CsState,
    ) {
        let c_code = self.get_c_code(rd);
        let mut mi = Microinstruction::new(cs_state.next_addr());
        (mi.a, mi.immediate) = self.val_a_code(rs1);
        mi.c_bus = c_code;
        mi.b = Microinstruction::NO_B;

        match op {
            SingleOperandOpcode::Lui => mi.alu = 0b000011000,
            SingleOperandOpcode::Not => mi.alu = 0b000011010,
            SingleOperandOpcode::Sll => mi.alu = 0b100011000,
            SingleOperandOpcode::Sra => mi.alu = 0b010011000,
            SingleOperandOpcode::Sla => mi.alu = 0b110011000,
            SingleOperandOpcode::Mov => mi.alu = 0b000011000,
        }
        cs_state.add_instr(mi.get());
    }

    fn eval_double_op_inst(
        &mut self,
        opcode: &DoubleOperandOpcode,
        rd: &Vec<Rc<Register>>,
        rs1: &Rc<Register>,
        rs2: &Value,
        cs_state: &mut CsState,
    ) {
        let c_code = self.get_c_code(rd);
        match opcode {
            DoubleOperandOpcode::Add | DoubleOperandOpcode::Addi => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b000111100;
                mi.a = self.reg_a_code(rs1.as_ref());
                (mi.b, mi.immediate) = self.val_b_code(rs2);
                cs_state.add_instr(mi.get());
            }
            DoubleOperandOpcode::Sub | DoubleOperandOpcode::Subi => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b000111111;
                mi.b = self.reg_b_code(rs1.as_ref());
                (mi.a, mi.immediate) = self.val_a_code(rs2);
                cs_state.add_instr(mi.get());
            }
            DoubleOperandOpcode::And | DoubleOperandOpcode::Andi => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b000011000;
                mi.a = self.reg_a_code(rs1.as_ref());
                (mi.b, mi.immediate) = self.val_b_code(rs2);
                cs_state.add_instr(mi.get());
            }
            DoubleOperandOpcode::Or | DoubleOperandOpcode::Ori => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b000011100;
                mi.a = self.reg_a_code(rs1.as_ref());
                (mi.b, mi.immediate) = self.val_b_code(rs2);
                cs_state.add_instr(mi.get());
            }
            DoubleOperandOpcode::Mul => {
                // Temp registers usage:
                // - T0: gonna store the min(rs1, rs2)
                // - T1: gonna store the max(rs1, rs2)
                let branched_addr = cs_state.next_addr() | 0b100000000;

                let mut mi = Microinstruction::new(cs_state.next_addr());
                // JUMP if rs1 > rs2
                mi.jam = 0b010;
                mi.alu = 0b000111111;
                mi.a = self.reg_a_code(rs1);
                (mi.b, mi.immediate) = self.val_b_code(rs2);
                cs_state.add_instr(mi.get());

                // HAS NOT JUMPED, therefore rs1 <= rs2
                // mv t0 <- rs1
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.alu = 0b000011000;
                mi.c_bus = self.get_c_code(&vec![Rc::new(Register::T0)]);
                mi.a = self.reg_a_code(rs1);
                cs_state.add_instr(mi.get());
                // mv t1, t2, rd <- rs2
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.alu = 0b000011000;
                mi.c_bus =
                    self.get_c_code(&vec![Rc::new(Register::T1), Rc::new(Register::T2)]) | c_code;
                mi.a = match rs2 {
                    Value::Reg(r) => self.reg_a_code(r),
                    Value::Immediate(_) => unreachable!("Should't receive a immediate arg."),
                    _ => unreachable!("Should't receive a label arg."),
                };
                let loop_addr = cs_state.next_addr();
                cs_state.add_instr(mi.get());

                // Intersection between the cases: t1 + .. + t1, t0-times
                // t0 <- t0 - 1 (special case because we subtract 1 without ussing immediate)
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.jam = 0b001;
                mi.alu = 0b000110110;
                mi.c_bus = self.get_c_code(&vec![Rc::new(Register::T0)]);
                mi.b = self.reg_b_code(&Register::T0);
                cs_state.add_instr(mi.get());
                // add t1, rd <- t1 + t2
                let mut mi = Microinstruction::new(loop_addr);
                mi.c_bus = self.get_c_code(&vec![Rc::new(Register::T1)]) | c_code;
                mi.alu = 0b000111100;
                mi.a = self.reg_a_code(&Register::T1);
                mi.b = self.reg_b_code(&Register::T2);
                cs_state.add_instr(mi.get());

                // HAS JUMPED, therefore rs1 > rs2
                // mv t0 <- rs2
                let mut mi = Microinstruction::new(cs_state.next_addr() - 1);
                mi.alu = 0b000011000;
                mi.c_bus = self.get_c_code(&vec![Rc::new(Register::T0)]);
                mi.a = match rs2 {
                    Value::Reg(r) => self.reg_a_code(r),
                    Value::Immediate(_) => unreachable!("Should't receive a immediate arg."),
                    _ => unreachable!("Should't receive a label arg."),
                };
                cs_state.add_complex_instr(|cs, addr| {
                    cs.set_word(branched_addr, mi.get());
                    addr
                });
                // mv t1, t2, rd <- rs1
                let mut mi = Microinstruction::new(loop_addr);
                mi.alu = 0b000011000;
                mi.c_bus =
                    self.get_c_code(&vec![Rc::new(Register::T1), Rc::new(Register::T2)]) | c_code;
                mi.a = self.reg_a_code(rs1);
                cs_state.add_instr(mi.get());

                let jal = Microinstruction::new(cs_state.addr());
                cs_state.set_instr((loop_addr + 1) | 0b100000000, jal.get())
            }
            DoubleOperandOpcode::Xor | DoubleOperandOpcode::Xori => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b001001100;
                mi.a = self.reg_a_code(rs1.as_ref());
                (mi.b, mi.immediate) = self.val_b_code(rs2);
                cs_state.add_instr(mi.get());
            }
            DoubleOperandOpcode::Mul2 | DoubleOperandOpcode::Muli => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b001011100;
                mi.a = self.reg_a_code(rs1.as_ref());
                (mi.b, mi.immediate) = self.val_b_code(rs2);
                cs_state.add_instr(mi.get());
            }
            DoubleOperandOpcode::Div | DoubleOperandOpcode::Divi => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b001101100;
                mi.a = self.reg_a_code(rs1.as_ref());
                (mi.b, mi.immediate) = self.val_b_code(rs2);
                cs_state.add_instr(mi.get());
            }
            DoubleOperandOpcode::Mod | DoubleOperandOpcode::Modi => {
                let mut mi = Microinstruction::new(cs_state.next_addr());
                mi.c_bus = c_code;
                mi.alu = 0b001111100;
                mi.a = self.reg_a_code(rs1.as_ref());
                (mi.b, mi.immediate) = self.val_b_code(rs2);
                cs_state.add_instr(mi.get());
            }
        }
    }

    /// Returns a pair of (A bus code, Immediate).
    fn val_a_code(&self, val: &Value) -> (u8, u8) {
        match val {
            Value::Reg(r) => (self.reg_a_code(r), 0),
            Value::Immediate(imm) => (Microinstruction::IMM_A, *imm),
            Value::Label(l) => (
                Microinstruction::IMM_A,
                *self.values.get(l.as_ref()).expect("Should be defined before"),
            ),
        }
    }

    /// Returns a pair of (B bus code, Immediate).
    fn val_b_code(&self, val: &Value) -> (u8, u8) {
        match val {
            Value::Reg(r) => (self.reg_b_code(r), 0),
            Value::Immediate(imm) => (Microinstruction::IMM_B, *imm),
            Value::Label(l) => (
                Microinstruction::IMM_B,
                *self.values.get(l.as_ref()).expect("Should be defined before"),
            ),
        }
    }

    fn reg_a_code(&self, reg: &Register) -> u8 {
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

    pub fn reg_b_code(&self, reg: &Register) -> u8 {
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
    fn get_c_code(&self, regs: &Vec<Rc<Register>>) -> u32 {
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
    pub curr_addr: u16,
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
        self.builder.set_word(self.curr_addr, inst);
        self.curr_addr = self.next_addr();
    }

    pub fn set_instr(&mut self, addr: u16, inst: u64) {
        self.builder.set_word(addr, inst);
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

#[derive(Clone)]
struct Microinstruction {
    pub next: u16,
    pub jam: u8,
    pub alu: u16,
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

        mi <<= 9;
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
        assert_eq!(
            0b10001000000000110001,
            AsmEvaluator::new().get_c_code(&regs)
        );
    }

    #[test]
    fn microinstruction() {
        let mut mi = Microinstruction::new(0);
        mi.next = 0b000000001;
        mi.jam = 0b000;
        mi.alu = 0b000111100;
        mi.c_bus = 0b00001100000000000000;
        mi.mem = 000;
        mi.a = 0b01011;
        mi.b = 0b01100;
        mi.immediate = 0b00000000;

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_000111100_00001100000000000000_000_01011_01100_00000000;

        assert_eq!(expected, mi.get());
    }

    #[test]
    fn basic_add() {
        // add a0, a1 <- t0, a3
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Add,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Reg(Rc::new(Register::A3)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_000111100_00000000000000001100_000_01010_10011_00000000;

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
                DoubleOperandOpcode::Addi,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Immediate(5),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_000111100_00000000000000001100_000_01010_00011_00000101;

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
                DoubleOperandOpcode::Sub,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Reg(Rc::new(Register::A3)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_000111111_00000000000000001100_000_11000_00101_00000000;

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
                DoubleOperandOpcode::Subi,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Immediate(7),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_000111111_00000000000000001100_000_01000_00101_00000111;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_xor() {
        // xor a0, a1 <- t0, a3
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Xor,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Reg(Rc::new(Register::A3)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_001001100_00000000000000001100_000_01010_10011_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_xori() {
        // muli a0, a1 <- t0, 5
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Xori,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Immediate(5),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_001001100_00000000000000001100_000_01010_00011_00000101;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_mul2() {
        // mul2 a0, a1 <- t0, a3
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Mul2,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Reg(Rc::new(Register::A3)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_001011100_00000000000000001100_000_01010_10011_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_muli() {
        // muli a0, a1 <- t0, 5
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Muli,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Immediate(5),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_001011100_00000000000000001100_000_01010_00011_00000101;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_div() {
        // mul2 a0, a1 <- t0, a3
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Div,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Reg(Rc::new(Register::A3)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_001101100_00000000000000001100_000_01010_10011_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_divi() {
        // muli a0, a1 <- t0, 5
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Divi,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Immediate(5),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_001101100_00000000000000001100_000_01010_00011_00000101;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_mod() {
        // mul2 a0, a1 <- t0, a3
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Mod,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Reg(Rc::new(Register::A3)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_001111100_00000000000000001100_000_01010_10011_00000000;

        assert_eq!(firmware[0], expected);
        assert_eq!(firmware[1], Microinstruction::HALT);
        for i in 2..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn test_modi() {
        // muli a0, a1 <- t0, 5
        let instructions = vec![
            Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Modi,
                vec![Rc::new(Register::A0), Rc::new(Register::A1)],
                Rc::new(Register::T0),
                Value::Immediate(5),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();
        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_001111100_00000000000000001100_000_01010_00011_00000101;

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
                SingleOperandOpcode::Lui,
                vec![Rc::new(Register::A3)],
                Value::Immediate(1),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_000011000_00000000000000000001_000_01000_11111_00000001;

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
                SingleOperandOpcode::Mov,
                vec![Rc::new(Register::A3)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_000011000_00000000000000000001_000_00000_11111_00000000;

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
                SingleOperandOpcode::Not,
                vec![Rc::new(Register::A3), Rc::new(Register::A2)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_000011010_00000000000000000011_000_00000_11111_00000000;

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
                SingleOperandOpcode::Sll,
                vec![Rc::new(Register::A3), Rc::new(Register::A2)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_100011000_00000000000000000011_000_00000_11111_00000000;

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
                SingleOperandOpcode::Sla,
                vec![Rc::new(Register::A3), Rc::new(Register::A2)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_110011000_00000000000000000011_000_00000_11111_00000000;

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
                SingleOperandOpcode::Sra,
                vec![Rc::new(Register::A3), Rc::new(Register::A2)],
                Value::Reg(Rc::new(Register::Mdr)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = 0b000000001_000_010011000_00000000000000000011_000_00000_11111_00000000;

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
                DoubleOperandOpcode::Mul,
                vec![Rc::new(Register::A0)],
                Rc::new(Register::A1),
                Value::Reg(Rc::new(Register::A2)),
            ),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let seg = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![seg]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        let no_branch_mcode: Vec<u64> = vec![
            // JUMP if a1 > a2 (a2 - a1 < 0)
            0b000000001_010_000111111_00000000000000000000_000_10110_10010_00000000,
            // CASE 1 (has not branched)
            // Copy a1 into t0
            0b000000010_000_000011000_00000100000000000000_000_10110_11111_00000000,
            // Copy a2 into t1, t2, a0
            0b000000011_000_000011000_00000011000000001000_000_10111_11111_00000000,
            // Intersection between the cases: r13 + .. + r13, r14-times
            // t0 <- t0 - 1 or JUMP if (t0 - 1) = 0
            0b000000100_001_000110110_00000100000000000000_000_11111_00101_00000000,
            // a0, t1 <- t1 + t2
            0b000000011_000_000111100_00000010000000001000_000_01011_00111_00000000,
            // CASE 2.1
            // Copy a1 into t1, t2, a0 and go to loop
            0b000000011_000_000011000_00000011000000001000_000_10110_11111_00000000,
            Microinstruction::HALT,
        ];

        let brached_mcode: Vec<u64> = vec![
            // CASE 2: has branched
            // Copy a2 into t0 as go to CASE 2.1
            0b000000101_000_000011000_00000100000000000000_000_10111_11111_00000000,
        ];

        for (addr, &mi) in no_branch_mcode.iter().enumerate() {
            // println!("addr:     {:09b}", addr);
            // eprintln!("expected: {:061b}", mi);
            // eprintln!("got:      {:061b}", firmware[addr]);
            assert_eq!(mi, firmware[addr]);
        }

        for (addr, &mi) in brached_mcode.iter().enumerate() {
            let addr = addr + 0b100000001;
            // println!("addr:     {:09b}", addr);
            // eprintln!("expected: {:061b}", mi);
            // eprintln!("got:      {:061b}", firmware[addr]);
            assert_eq!(mi, firmware[addr]);
        }
        let jal = 0b000000110_000_000000000_00000000000000000000_000_11111_11111_00000000;
        assert_eq!(jal, firmware[0b100000100]);
    }

    #[test]
    fn read() {
        /*
         * .data
         *   tubias: .word 777
         *   gepeto: .word 42069
         *   tubias_addr: .byte 0
         * .text
         *   main:
         *    read  a1, a2, a3 <- 0
         *    read a1 <- gepeto
         *    read a1 <- tubias_addr
         *    halt
         */
        let program = Program {
            sections: vec![
                Sections::new_data_section(vec![
                    Sections::new_data_writed(DataKind::Word(777), Rc::from("tubias")),
                    Sections::new_data_writed(DataKind::Word(42069), Rc::from("gepeto")),
                    Sections::new_data_writed(DataKind::Byte(0), Rc::from("tubias_addr")),
                ]),
                Sections::new_text_section(vec![TextSegment::new_labeled_section(
                    Rc::from("main"),
                    vec![
                        Instruction::new_read_instruction(
                            ImmediateOrLabel::Immediate(0),
                            vec![
                                Rc::new(Register::A1),
                                Rc::new(Register::A2),
                                Rc::new(Register::A3),
                            ],
                        ),
                        Instruction::new_read_instruction(
                            ImmediateOrLabel::Label(Rc::from("gepeto")),
                            vec![Rc::new(Register::A1)],
                        ),
                        Instruction::new_read_instruction(
                            ImmediateOrLabel::Label(Rc::from("tubias_addr")),
                            vec![Rc::new(Register::A1)],
                        ),
                        Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
                    ],
                )]),
            ],
            errors: vec![],
        };
        let mut eval = AsmEvaluator::new();
        let (cs, _) = eval.eval_program(program).unwrap();
        let firmware = cs.firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = [
            // mar <- 0 and READ
            0b000000001_000_000011000_01000000000000000000_010_01000_11111_00000000,
            // a1, a2, a3 <- mdr
            0b000000010_000_000011000_00000000000000000111_000_00000_11111_00000000,
            // mar <- 1 and READ
            0b000000011_000_000011000_01000000000000000000_010_01000_11111_00000001,
            // a1 <- mdr
            0b000000100_000_000011000_00000000000000000100_000_00000_11111_00000000,
            // mar <- 0 and READ
            0b000000101_000_000011000_01000000000000000000_010_01000_11111_00000000,
            // a1 <- mdr
            0b000000110_000_000011000_00000000000000000100_000_00000_11111_00000000,
            Microinstruction::HALT,
        ];

        for (addr, mi) in expected.iter().enumerate() {
            assert_eq!(firmware[addr], *mi);
        }

        for i in expected.len()..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn write() {
        /*
         * .data
         *   tubias: .word 777
         *   gepeto: .word 42069
         *   tubias_addr: .byte 0
         * .text
         *   main:
         *    write  0 <- a3
         *    write gepeto <- a2
         *    write tubias_addr <- a1
         *    halt
         */
        let program = Program {
            sections: vec![
                Sections::new_data_section(vec![
                    Sections::new_data_writed(DataKind::Word(777), Rc::from("tubias")),
                    Sections::new_data_writed(DataKind::Word(42069), Rc::from("gepeto")),
                    Sections::new_data_writed(DataKind::Byte(0), Rc::from("tubias_addr")),
                ]),
                Sections::new_text_section(vec![TextSegment::new_labeled_section(
                    Rc::from("main"),
                    vec![
                        Instruction::new_write_instruction(
                            ImmediateOrLabel::Immediate(0),
                            Rc::new(Register::A3),
                        ),
                        Instruction::new_write_instruction(
                            ImmediateOrLabel::Label(Rc::from("gepeto")),
                            Rc::new(Register::A2),
                        ),
                        Instruction::new_write_instruction(
                            ImmediateOrLabel::Label(Rc::from("tubias_addr")),
                            Rc::new(Register::A1),
                        ),
                        Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
                    ],
                )]),
            ],
            errors: vec![],
        };
        let mut eval = AsmEvaluator::new();
        let (cs, _) = eval.eval_program(program).unwrap();
        let firmware = cs.firmware();

        #[allow(clippy::unusual_byte_groupings)]
        let expected = [
            // mdr <- a3
            0b000000001_000_000011000_10000000000000000000_000_11000_11111_00000000,
            // mar <- 0 and WRITE
            0b000000010_000_000011000_01000000000000000000_100_01000_11111_00000000,
            // mdr <- a2
            0b000000011_000_000011000_10000000000000000000_000_10111_11111_00000000,
            // mar <- address of gepeto that is 1 and WRITE
            0b000000100_000_000011000_01000000000000000000_100_01000_11111_00000001,
            // mdr <- a1
            0b000000101_000_000011000_10000000000000000000_000_10110_11111_00000000,
            // mar <- tubias_addr that is 0 and WRITE
            0b000000110_000_000011000_01000000000000000000_100_01000_11111_00000000,
            Microinstruction::HALT,
        ];

        for (addr, mi) in expected.iter().enumerate() {
            assert_eq!(firmware[addr], *mi);
        }

        for i in expected.len()..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn jal() {
        let instructions = vec![
            Instruction::new_jal_instruction(Rc::from("tubias")),
            Instruction::new_no_operand_instruction(NoOperandOpcode::Halt),
        ];
        let tubias = TextSegment::new_labeled_section(
            "tubias".into(),
            vec![Instruction::new_no_operand_instruction(
                NoOperandOpcode::Halt,
            )],
        );
        let main = TextSegment::new_labeled_section("main".into(), instructions);
        let secs = Sections::new_text_section(vec![tubias, main]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        let expected = [
            Microinstruction::HALT,
            0b000000000_000_000000000_00000000000000000000_000_11111_11111_00000000,
            Microinstruction::HALT,
        ];

        for (addr, mi) in expected.iter().enumerate() {
            assert_eq!(firmware[addr], *mi);
        }

        for i in expected.len()..=255 {
            assert_eq!(firmware[i], 0);
        }
    }

    #[test]
    fn beq() {
        let done = TextSegment::new_labeled_section(
            "done".into(),
            vec![Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Add,
                vec![Rc::new(Register::A0)],
                Rc::new(Register::A1),
                Value::Reg(Rc::new(Register::A2)),
            )],
        );
        let main = TextSegment::new_labeled_section(
            "main".into(),
            vec![
                Instruction::new_double_operand_instruction(
                    DoubleOperandOpcode::Add,
                    vec![Rc::new(Register::A0)],
                    Rc::new(Register::A1),
                    Value::Reg(Rc::new(Register::A2)),
                ),
                Instruction::new_branch_instruction(
                    BranchOp::Beq,
                    Rc::new(Register::A1),
                    Rc::new(Register::A2),
                    Rc::from("done"),
                ),
            ],
        );
        let secs = Sections::new_text_section(vec![done, main]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        let no_branch_mcode: Vec<u64> = vec![
            // add a0 <- a1, a2
            0b000000001_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // add a0 <- a1, a2
            0b000000010_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // beq a1 == a2, done
            0b000000011_001_000111111_00000000000000000000_000_10110_10010_00000000,
        ];

        let branched_mcode = 0b000000000_000_00000000_00000000000000000000_000_11111_11111_00000000;

        for (addr, &mi) in no_branch_mcode.iter().enumerate() {
            assert_eq!(mi, firmware[addr]);
        }

        let branched_addr = 3 | 0b100000000;
        assert_eq!(branched_mcode, firmware[branched_addr]);
    }

    #[test]
    fn bne() {
        let done = TextSegment::new_labeled_section(
            "done".into(),
            vec![Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Add,
                vec![Rc::new(Register::A0)],
                Rc::new(Register::A1),
                Value::Reg(Rc::new(Register::A2)),
            )],
        );
        let main = TextSegment::new_labeled_section(
            "main".into(),
            vec![
                Instruction::new_double_operand_instruction(
                    DoubleOperandOpcode::Add,
                    vec![Rc::new(Register::A0)],
                    Rc::new(Register::A1),
                    Value::Reg(Rc::new(Register::A2)),
                ),
                Instruction::new_branch_instruction(
                    BranchOp::Bne,
                    Rc::new(Register::A2),
                    Rc::new(Register::A3),
                    Rc::from("done"),
                ),
            ],
        );
        let secs = Sections::new_text_section(vec![done, main]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        let no_branch_mcode: Vec<u64> = vec![
            // add a0 <- a1, a2
            0b000000001_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // add a0 <- a1, a2
            0b000000010_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // bne a2, a3, done
            0b000000011_010_000111111_00000000000000000000_000_10111_10011_00000000,
            0b000000100_010_000111111_00000000000000000000_000_11000_10010_00000000,
        ];

        let branched_mcode = vec![
            0b000000000_000_000000000_00000000000000000000_000_11111_11111_00000000,
            0b000000000_000_000000000_00000000000000000000_000_11111_11111_00000000,
        ];

        for (addr, &mi) in no_branch_mcode.iter().enumerate() {
            assert_eq!(mi, firmware[addr]);
        }

        let branched_addr = 3 | 0b100000000;
        assert_eq!(branched_mcode[0], firmware[branched_addr]);
        assert_eq!(branched_mcode[1], firmware[branched_addr + 1]);
    }

    #[test]
    fn blt() {
        let done = TextSegment::new_labeled_section(
            "done".into(),
            vec![Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Add,
                vec![Rc::new(Register::A0)],
                Rc::new(Register::A1),
                Value::Reg(Rc::new(Register::A2)),
            )],
        );
        let main = TextSegment::new_labeled_section(
            "main".into(),
            vec![
                Instruction::new_double_operand_instruction(
                    DoubleOperandOpcode::Add,
                    vec![Rc::new(Register::A0)],
                    Rc::new(Register::A1),
                    Value::Reg(Rc::new(Register::A2)),
                ),
                Instruction::new_branch_instruction(
                    BranchOp::Blt,
                    Rc::new(Register::A2),
                    Rc::new(Register::A3),
                    Rc::from("done"),
                ),
            ],
        );
        let secs = Sections::new_text_section(vec![done, main]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        let no_branch_mcode: Vec<u64> = vec![
            // add a0 <- a1, a2
            0b000000001_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // add a0 <- a1, a2
            0b000000010_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // blt a2, a3, done
            0b000000011_010_000111111_00000000000000000000_000_11000_10010_00000000,
        ];

        let branched_mcode =
            0b000000000_000_000000000_00000000000000000000_000_11111_11111_00000000;

        for (addr, &mi) in no_branch_mcode.iter().enumerate() {
            assert_eq!(mi, firmware[addr]);
        }

        let branched_addr = 3 | 0b100000000;
        assert_eq!(branched_mcode, firmware[branched_addr]);
    }

    #[test]
    fn bgt() {
        let done = TextSegment::new_labeled_section(
            "done".into(),
            vec![Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Add,
                vec![Rc::new(Register::A0)],
                Rc::new(Register::A1),
                Value::Reg(Rc::new(Register::A2)),
            )],
        );
        let main = TextSegment::new_labeled_section(
            "main".into(),
            vec![
                Instruction::new_double_operand_instruction(
                    DoubleOperandOpcode::Add,
                    vec![Rc::new(Register::A0)],
                    Rc::new(Register::A1),
                    Value::Reg(Rc::new(Register::A2)),
                ),
                Instruction::new_branch_instruction(
                    BranchOp::Bgt,
                    Rc::new(Register::A2),
                    Rc::new(Register::A3),
                    Rc::from("done"),
                ),
            ],
        );
        let secs = Sections::new_text_section(vec![done, main]);
        let firmware = AsmEvaluator::new().eval(&secs).firmware();

        let no_branch_mcode: Vec<u64> = vec![
            // add a0 <- a1, a2
            0b000000001_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // add a0 <- a1, a2
            0b000000010_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // bgt a2, a3, done
            0b000000011_010_000111111_00000000000000000000_000_10111_10011_00000000,
        ];

        let branched_mcode =
            0b000000000_000_000000000_00000000000000000000_000_11111_11111_00000000;

        for (addr, &mi) in no_branch_mcode.iter().enumerate() {
            assert_eq!(mi, firmware[addr]);
        }

        let branched_addr = 3 | 0b100000000;
        assert_eq!(branched_mcode, firmware[branched_addr]);
    }

    #[test]
    fn resolve_unreachable() {
        let main = TextSegment::new_labeled_section(
            "main".into(),
            vec![
                Instruction::new_double_operand_instruction(
                    DoubleOperandOpcode::Add,
                    vec![Rc::new(Register::A0)],
                    Rc::new(Register::A1),
                    Value::Reg(Rc::new(Register::A2)),
                ),
                Instruction::new_branch_instruction(
                    BranchOp::Beq,
                    Rc::new(Register::A2),
                    Rc::new(Register::A3),
                    Rc::from("unresolved"),
                ),
            ],
        );
        let unresolved = TextSegment::new_labeled_section(
            "unresolved".into(),
            vec![Instruction::new_double_operand_instruction(
                DoubleOperandOpcode::Add,
                vec![Rc::new(Register::A0)],
                Rc::new(Register::A1),
                Value::Reg(Rc::new(Register::A2)),
            )],
        );

        let mut evaluator = AsmEvaluator::new();
        let mut state = CsState::new();
        evaluator.eval_txt_seg(&main, &mut state);
        evaluator.eval_txt_seg(&unresolved, &mut state);
        evaluator.resolve_unreachable(&mut state);

        let firmware = state.build_cs().firmware();

        let no_branch_mcode: Vec<u64> = vec![
            // add a0 <- a1, a2
            0b000000001_000_000111100_00000000000000001000_000_10110_10010_00000000,
            // beq a2, a3, done
            0b000000010_001_000111111_00000000000000000000_000_10111_10011_00000000,
            // add a0 <- a1, a2
            0b000000011_000_000111100_00000000000000001000_000_10110_10010_00000000,
        ];

        let branched_mcode =
            0b000000010_000_000000000_00000000000000000000_000_11111_11111_00000000;

        for (addr, &mi) in no_branch_mcode.iter().enumerate() {
            assert_eq!(mi, firmware[addr]);
        }

        let branched_addr = 2 | 0b100000000;
        assert_eq!(branched_mcode, firmware[branched_addr]);
    }
}
