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
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
    Pc,
}

#[derive(Debug, PartialEq)]
pub enum Opcode {
    Add,
    Addi,
    And,
    Andi,
    Auipc,
    Beq,
    Bge,
    Bgeu,
    Blt,
    Bltu,
    Bne,
    Jal,
    Jalr,
    Lb,
    Lbu,
    Lh,
    Lhu,
    Lui,
    Lw,
    Or,
    Ori,
    Sb,
    Sh,
    Sll,
    Slli,
    Slt,
    Slti,
    Sltiu,
    Sltu,
    Sra,
    Srai,
    Srl,
    Srli,
    Sub,
    Sw,
    Xor,
    Xori,
    Ecall,
    Ebreak,
    Fence,
    Fencei,
    Mret,
    Sfence,
    Sfencevma,
}

#[derive(Debug, PartialEq)]
pub enum AsmToken {
    Identifier(String),
    Reg(Register),
    Opcode(Opcode),
    Comma,
    Colon,
    Illegal,
    Eof,
}

impl AsmToken {
    pub fn name_to_tok(input: String) -> AsmToken {
        match input.as_str() {
            //OPCODES
            "add" => AsmToken::Opcode(Opcode::Add),
            "addi" => AsmToken::Opcode(Opcode::Addi),
            "and" => AsmToken::Opcode(Opcode::And),
            "andi" => AsmToken::Opcode(Opcode::Andi),
            "auipc" => AsmToken::Opcode(Opcode::Auipc),
            "beq" => AsmToken::Opcode(Opcode::Beq),
            "bge" => AsmToken::Opcode(Opcode::Bge),
            "bgeu" => AsmToken::Opcode(Opcode::Bgeu),
            "blt" => AsmToken::Opcode(Opcode::Blt),
            "bltu" => AsmToken::Opcode(Opcode::Bltu),
            "bne" => AsmToken::Opcode(Opcode::Bne),
            "jal" => AsmToken::Opcode(Opcode::Jal),
            "jalr" => AsmToken::Opcode(Opcode::Jalr),
            "lb" => AsmToken::Opcode(Opcode::Lb),
            "lbu" => AsmToken::Opcode(Opcode::Lbu),
            "lh" => AsmToken::Opcode(Opcode::Lh),
            "lhu" => AsmToken::Opcode(Opcode::Lhu),
            "lui" => AsmToken::Opcode(Opcode::Lui),
            "lw" => AsmToken::Opcode(Opcode::Lw),
            "or" => AsmToken::Opcode(Opcode::Or),
            "ori" => AsmToken::Opcode(Opcode::Ori),
            "sb" => AsmToken::Opcode(Opcode::Sb),
            "sh" => AsmToken::Opcode(Opcode::Sh),
            "sll" => AsmToken::Opcode(Opcode::Sll),
            "slli" => AsmToken::Opcode(Opcode::Slli),
            "slt" => AsmToken::Opcode(Opcode::Slt),
            "slti" => AsmToken::Opcode(Opcode::Slti),
            "sltiu" => AsmToken::Opcode(Opcode::Sltiu),
            "sltu" => AsmToken::Opcode(Opcode::Sltu),
            "sra" => AsmToken::Opcode(Opcode::Sra),
            "srai" => AsmToken::Opcode(Opcode::Srai),
            "srl" => AsmToken::Opcode(Opcode::Srl),
            "srli" => AsmToken::Opcode(Opcode::Srli),
            "sub" => AsmToken::Opcode(Opcode::Sub),
            "sw" => AsmToken::Opcode(Opcode::Sw),
            "xor" => AsmToken::Opcode(Opcode::Xor),
            "xori" => AsmToken::Opcode(Opcode::Xori),
            "ecall" => AsmToken::Opcode(Opcode::Ecall),
            "ebreak" => AsmToken::Opcode(Opcode::Ebreak),
            //REGISTERS
            "zero" | "x0" => AsmToken::Reg(Register::Zero),
            "ra" | "x1" => AsmToken::Reg(Register::Ra),
            "sp" | "x2" => AsmToken::Reg(Register::Sp),
            "gp" | "x3" => AsmToken::Reg(Register::Gp),
            "tp" | "x4" => AsmToken::Reg(Register::Tp),
            "t0" | "x5" => AsmToken::Reg(Register::T0),
            "t1" | "x6" => AsmToken::Reg(Register::T1),
            "t2" | "x7" => AsmToken::Reg(Register::T2),
            "s0" | "fp" | "x8" => AsmToken::Reg(Register::S0),
            "s1" | "x9" => AsmToken::Reg(Register::S1),
            "a0" | "x10" => AsmToken::Reg(Register::A0),
            "a1" | "x11" => AsmToken::Reg(Register::A1),
            "a2" | "x12" => AsmToken::Reg(Register::A2),
            "a3" | "x13" => AsmToken::Reg(Register::A3),
            "a4" | "x14" => AsmToken::Reg(Register::A4),
            "a5" | "x15" => AsmToken::Reg(Register::A5),
            "a6" | "x16" => AsmToken::Reg(Register::A6),
            "a7" | "x17" => AsmToken::Reg(Register::A7),
            "s2" | "x18" => AsmToken::Reg(Register::S2),
            "s3" | "x19" => AsmToken::Reg(Register::S3),
            "s4" | "x20" => AsmToken::Reg(Register::S4),
            "s5" | "x21" => AsmToken::Reg(Register::S5),
            "s6" | "x22" => AsmToken::Reg(Register::S6),
            "s7" | "x23" => AsmToken::Reg(Register::S7),
            "s8" | "x24" => AsmToken::Reg(Register::S8),
            "s9" | "x25" => AsmToken::Reg(Register::S9),
            "s10" | "x26" => AsmToken::Reg(Register::S10),
            "s11" | "x27" => AsmToken::Reg(Register::S11),
            "t3" | "x28" => AsmToken::Reg(Register::T3),
            "t4" | "x29" => AsmToken::Reg(Register::T4),
            "t5" | "x30" => AsmToken::Reg(Register::T5),
            "t6" | "x31" => AsmToken::Reg(Register::T6),
            _ => AsmToken::Identifier(input),
        }
    }
}
