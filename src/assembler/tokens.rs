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
    Mbru,
    Mbr2,
    Mbr2u,
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
    Lui,
    Addi,
    Subi,
    Slti,
    Andi,
    Ori,
    // Integer Register-Register Operations
    Add,
    Sub,
    Slt,
    And,
    Or,
    Not,
    Mov,
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
    Bgt,
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
            "lui" => AsmToken::Opcode(Rc::new(Opcode::Lui)),
            "addi" => AsmToken::Opcode(Rc::new(Opcode::Addi)),
            "subi" => AsmToken::Opcode(Rc::new(Opcode::Subi)),
            "slti" => AsmToken::Opcode(Rc::new(Opcode::Slti)),
            "andi" => AsmToken::Opcode(Rc::new(Opcode::Andi)),
            "ori" => AsmToken::Opcode(Rc::new(Opcode::Ori)),
            "add" => AsmToken::Opcode(Rc::new(Opcode::Add)),
            "sub" => AsmToken::Opcode(Rc::new(Opcode::Sub)),
            "slt" => AsmToken::Opcode(Rc::new(Opcode::Slt)),
            "and" => AsmToken::Opcode(Rc::new(Opcode::And)),
            "or" => AsmToken::Opcode(Rc::new(Opcode::Or)),
            "not" => AsmToken::Opcode(Rc::new(Opcode::Not)),
            "mov" => AsmToken::Opcode(Rc::new(Opcode::Mov)),
            "sll" => AsmToken::Opcode(Rc::new(Opcode::Sll)),
            "sra" => AsmToken::Opcode(Rc::new(Opcode::Sra)),
            "sla" => AsmToken::Opcode(Rc::new(Opcode::Sla)),
            "nop" => AsmToken::Opcode(Rc::new(Opcode::Nop)),
            "jal" => AsmToken::Opcode(Rc::new(Opcode::Jal)),
            "beq" => AsmToken::Opcode(Rc::new(Opcode::Beq)),
            "bne" => AsmToken::Opcode(Rc::new(Opcode::Bne)),
            "blt" => AsmToken::Opcode(Rc::new(Opcode::Blt)),
            "bgt" => AsmToken::Opcode(Rc::new(Opcode::Bgt)),
            "mul" => AsmToken::Opcode(Rc::new(Opcode::Mul)),
            "halt" => AsmToken::Opcode(Rc::new(Opcode::Halt)),
            "read" => AsmToken::Opcode(Rc::new(Opcode::Read)),
            "write" => AsmToken::Opcode(Rc::new(Opcode::Write)),

            //REGISTERS
            "mar" => AsmToken::Reg(Rc::new(Register::Mar)),
            "mdr" => AsmToken::Reg(Rc::new(Register::Mdr)),
            "pc" => AsmToken::Reg(Rc::new(Register::Pc)),
            "mbr" => AsmToken::Reg(Rc::new(Register::Mbr)),
            "mbru" => AsmToken::Reg(Rc::new(Register::Mbru)),
            "mbr2" => AsmToken::Reg(Rc::new(Register::Mbr2)),
            "mbr2u" => AsmToken::Reg(Rc::new(Register::Mbr2u)),
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
