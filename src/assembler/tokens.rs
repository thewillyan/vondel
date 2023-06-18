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
    Mar,
    Mdr,
    Mbr,
    Mbr2,
    Pc,
    Cpp,
    Lv,
    // General Purpose
    Ra,
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
    Addi,
    Slti,
    Andi,
    Ori,
    Xori,
    // Integer Register-Register Operations
    Add,
    Sub,
    Slt,
    And,
    Or,
    Not,
    // Shift left 1 bit
    Sll,
    Sla,
    Sra,
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
    Reg(Rc<Register>),
    Opcode(Rc<Opcode>),
    PseudoIns(PseudoInstruction),
    PseudoOp(Rc<PseudoOps>),
    Comma,
    Colon,
    Illegal,
    Eof,
    Assign,
}

impl AsmToken {
    pub fn name_to_tok(input: &str) -> AsmToken {
        match input {
            //PseudoOps
            pseudo if pseudo.starts_with('.') => match pseudo {
                ".data" => AsmToken::PseudoOp(Rc::new(PseudoOps::Data)),
                ".word" => AsmToken::PseudoOp(Rc::new(PseudoOps::Word)),
                ".byte" => AsmToken::PseudoOp(Rc::new(PseudoOps::Byte)),
                ".text" => AsmToken::PseudoOp(Rc::new(PseudoOps::Text)),
                ".global" => AsmToken::PseudoOp(Rc::new(PseudoOps::Global)),
                _ => AsmToken::Illegal,
            },

            //OPCODES
            "addi" => AsmToken::Opcode(Rc::new(Opcode::Addi)),
            "slti" => AsmToken::Opcode(Rc::new(Opcode::Slti)),
            "andi" => AsmToken::Opcode(Rc::new(Opcode::Andi)),
            "ori" => AsmToken::Opcode(Rc::new(Opcode::Ori)),
            "xori" => AsmToken::Opcode(Rc::new(Opcode::Xori)),
            "add" => AsmToken::Opcode(Rc::new(Opcode::Add)),
            "sub" => AsmToken::Opcode(Rc::new(Opcode::Sub)),
            "slt" => AsmToken::Opcode(Rc::new(Opcode::Slt)),
            "and" => AsmToken::Opcode(Rc::new(Opcode::And)),
            "or" => AsmToken::Opcode(Rc::new(Opcode::Or)),
            "not" => AsmToken::Opcode(Rc::new(Opcode::Not)),
            "sll" => AsmToken::Opcode(Rc::new(Opcode::Sll)),
            "sra" => AsmToken::Opcode(Rc::new(Opcode::Sra)),
            "sla" => AsmToken::Opcode(Rc::new(Opcode::Sla)),
            "nop" => AsmToken::Opcode(Rc::new(Opcode::Nop)),
            "jal" => AsmToken::Opcode(Rc::new(Opcode::Jal)),
            "beq" => AsmToken::Opcode(Rc::new(Opcode::Beq)),
            "bne" => AsmToken::Opcode(Rc::new(Opcode::Bne)),
            "blt" => AsmToken::Opcode(Rc::new(Opcode::Blt)),
            "bge" => AsmToken::Opcode(Rc::new(Opcode::Bge)),
            "mul" => AsmToken::Opcode(Rc::new(Opcode::Mul)),
            "halt" => AsmToken::Opcode(Rc::new(Opcode::Halt)),
            "read" => AsmToken::Opcode(Rc::new(Opcode::Read)),
            "write" => AsmToken::Opcode(Rc::new(Opcode::Write)),

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
            "mar" => AsmToken::Reg(Rc::new(Register::Mar)),
            "mdr" => AsmToken::Reg(Rc::new(Register::Mdr)),
            "pc" => AsmToken::Reg(Rc::new(Register::Pc)),
            "mbr" => AsmToken::Reg(Rc::new(Register::Mbr)),
            "mbr2" => AsmToken::Reg(Rc::new(Register::Mbr2)),
            "cpp" => AsmToken::Reg(Rc::new(Register::Cpp)),
            "lv" => AsmToken::Reg(Rc::new(Register::Lv)),

            // General Purpose
            "ra" => AsmToken::Reg(Rc::new(Register::Ra)),
            "t0" => AsmToken::Reg(Rc::new(Register::T0)),
            "t1" => AsmToken::Reg(Rc::new(Register::T1)),
            "t2" => AsmToken::Reg(Rc::new(Register::T2)),
            "t3" => AsmToken::Reg(Rc::new(Register::T3)),
            "s0" => AsmToken::Reg(Rc::new(Register::S0)),
            "s1" => AsmToken::Reg(Rc::new(Register::S1)),
            "s2" => AsmToken::Reg(Rc::new(Register::S2)),
            "s3" => AsmToken::Reg(Rc::new(Register::S3)),
            "s4" => AsmToken::Reg(Rc::new(Register::S4)),
            "s5" => AsmToken::Reg(Rc::new(Register::S5)),
            "s6" => AsmToken::Reg(Rc::new(Register::S6)),
            "a0" => AsmToken::Reg(Rc::new(Register::A0)),
            "a1" => AsmToken::Reg(Rc::new(Register::A1)),
            "a2" => AsmToken::Reg(Rc::new(Register::A2)),
            "a3" => AsmToken::Reg(Rc::new(Register::A3)),

            _ => AsmToken::Label(Rc::from(input)),
        }
    }
}
