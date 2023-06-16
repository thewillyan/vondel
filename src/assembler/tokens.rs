#[derive(Debug, PartialEq)]
pub struct TokWithCtx {
    pub tok: AsmToken,
    pub cur_line: usize,
    pub cur_column: usize,
}

impl TokWithCtx {
    pub fn new(tok: AsmToken, cur_line: usize, cur_column: usize) -> Self {
        TokWithCtx {
            tok,
            cur_line,
            cur_column,
        }
    }
}
//RISC-V ABI
#[derive(Debug, PartialEq)]
pub enum Register {
    Zero,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    S2,
    S3,
    S4,
    S5,
    S6,
    T3,
}

#[derive(Debug, PartialEq)]
pub enum Opcode {
    // Integer Register-Immediate Instructions
    Addi,
    Slti,
    Andi,
    Ori,
    Xori,
    Slli,
    Srli,
    Lui,
    Auipc,
    // Integer Register-Register Operations
    Add,
    Sub,
    Slt,
    And,
    Or,
    Sll,
    Srl,
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
    // Halt
    Halt,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum PseudoOps {
    Data,
    Word,
    Byte,
    Text,
    Global,
}

#[derive(Debug, PartialEq)]
pub enum AsmToken {
    Label(String),
    Reg(Register),
    Opcode(Opcode),
    PseudoIns(PseudoInstruction),
    PseudoOp(PseudoOps),
    Comma,
    Colon,
    Illegal,
    Eof,
}

impl AsmToken {
    pub fn name_to_tok(input: String) -> AsmToken {
        match input.as_str() {
            //PseudoOps
            pseudo if pseudo.starts_with('.') => match pseudo {
                ".data" => AsmToken::PseudoOp(PseudoOps::Data),
                ".word" => AsmToken::PseudoOp(PseudoOps::Word),
                ".byte" => AsmToken::PseudoOp(PseudoOps::Byte),
                ".text" => AsmToken::PseudoOp(PseudoOps::Text),
                ".global" => AsmToken::PseudoOp(PseudoOps::Global),
                _ => AsmToken::Illegal,
            },

            //OPCODES
            "addi" => AsmToken::Opcode(Opcode::Addi),
            "slti" => AsmToken::Opcode(Opcode::Slti),
            "andi" => AsmToken::Opcode(Opcode::Andi),
            "ori" => AsmToken::Opcode(Opcode::Ori),
            "xori" => AsmToken::Opcode(Opcode::Xori),
            "slli" => AsmToken::Opcode(Opcode::Slli),
            "srli" => AsmToken::Opcode(Opcode::Srli),
            "lui" => AsmToken::Opcode(Opcode::Lui),
            "auipc" => AsmToken::Opcode(Opcode::Auipc),
            "add" => AsmToken::Opcode(Opcode::Add),
            "sub" => AsmToken::Opcode(Opcode::Sub),
            "slt" => AsmToken::Opcode(Opcode::Slt),
            "and" => AsmToken::Opcode(Opcode::And),
            "or" => AsmToken::Opcode(Opcode::Or),
            "sll" => AsmToken::Opcode(Opcode::Sll),
            "srl" => AsmToken::Opcode(Opcode::Srl),
            "nop" => AsmToken::Opcode(Opcode::Nop),
            "jal" => AsmToken::Opcode(Opcode::Jal),
            "beq" => AsmToken::Opcode(Opcode::Beq),
            "bne" => AsmToken::Opcode(Opcode::Bne),
            "blt" => AsmToken::Opcode(Opcode::Blt),
            "bge" => AsmToken::Opcode(Opcode::Bge),
            "mul" => AsmToken::Opcode(Opcode::Mul),
            "halt" => AsmToken::Opcode(Opcode::Halt),

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
            "zero" | "x0" => AsmToken::Reg(Register::Zero),
            "t0" | "x1" => AsmToken::Reg(Register::T0),
            "t1" | "x2" => AsmToken::Reg(Register::T1),
            "t2" | "x3" => AsmToken::Reg(Register::T2),
            "s0" | "fp" | "x4" => AsmToken::Reg(Register::S0),
            "s1" | "x5" => AsmToken::Reg(Register::S1),
            "a0" | "x6" => AsmToken::Reg(Register::A0),
            "a1" | "x7" => AsmToken::Reg(Register::A1),
            "a2" | "x8" => AsmToken::Reg(Register::A2),
            "a3" | "x9" => AsmToken::Reg(Register::A3),
            "s2" | "x10" => AsmToken::Reg(Register::S2),
            "s3" | "x11" => AsmToken::Reg(Register::S3),
            "s4" | "x12" => AsmToken::Reg(Register::S4),
            "s5" | "x13" => AsmToken::Reg(Register::S5),
            "s6" | "x14" => AsmToken::Reg(Register::S6),
            "t3" | "x15" => AsmToken::Reg(Register::T3),
            _ => AsmToken::Label(input),
        }
    }
}
