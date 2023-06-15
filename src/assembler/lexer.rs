use std::{iter::Peekable, str::Chars};

use crate::assembler::tokens::AsmToken;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    cur_line: usize,
    cur_column: usize,
    cur_char: char,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut l = Lexer {
            chars: input.chars().peekable(),
            cur_line: 1,
            cur_column: 0,
            cur_char: '\0',
        };
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        match self.chars.next() {
            Some(c) => {
                self.cur_char = c;
                match c {
                    '\n' => {
                        self.cur_line += 1;
                        self.cur_column = 1;
                    }
                    '\t' => self.cur_column += 4,
                    _ => self.cur_column += 1,
                };
            }
            None => self.cur_char = '\0',
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.cur_char.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    fn ignore_comment(&mut self) {
        if self.cur_char == '#' || self.cur_char == ';' {
            self.read_char();
            while self.cur_char != '\0' && self.cur_char != '\n' {
                self.read_char();
            }
            self.skip_whitespace();
        }
    }

    fn read_identifier(&mut self) -> AsmToken {
        let mut ident = String::from(self.cur_char);
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.read_char();
            } else {
                break;
            }
        }
        AsmToken::name_to_tok(ident)
    }

    fn tokenizer(&mut self) -> AsmToken {
        match self.cur_char {
            ':' => AsmToken::Colon,
            ',' => AsmToken::Comma,
            c if c.is_alphabetic() => self.read_identifier(),
            '\0' => AsmToken::Eof,
            _ => AsmToken::Illegal,
        }
    }

    pub fn next_token(&mut self) -> AsmToken {
        self.skip_whitespace();
        self.ignore_comment();
        let tok = self.tokenizer();
        self.read_char();
        tok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_whitespace() {
        let input = "    \t\n tubias";
        let mut l = Lexer::new(input);
        assert_eq!(l.next_token(), AsmToken::Identifier("tubias".to_string()));
    }

    #[test]
    fn ignore_comments() {
        let input = "# this is a comment\n tubias ; another comment here \n another_tubias #comment until end";
        let mut l = Lexer::new(input);
        assert_eq!(l.next_token(), AsmToken::Identifier("tubias".to_string()));
        assert_eq!(
            l.next_token(),
            AsmToken::Identifier("another_tubias".to_string())
        );
        assert_eq!(l.next_token(), AsmToken::Eof);
    }

    #[test]
    fn get_registers() {
        use super::AsmToken::{Eof, Reg};
        use crate::assembler::tokens::Register::*;
        let input = r"
        zero ra sp gp tp t0 t1 t2 s0 s1 a0 a1 a2 a3 a4 a5
        a6 a7 s2 s3 s4 s5 s6 s7 s8 s9 s10 s11 t3 t4 t5 t6
        x0 x1 x2 x3 x4 x5 x6 x7 x8 x9 x10 x11 x12 x13 x14 x15
        x16 x17 x18 x19 x20 x21 x22 x23 x24 x25 x26 x27 x28 x29 x30 x31
";
        let mut l = Lexer::new(input);
        let toks = vec![
            Reg(Zero),
            Reg(Ra),
            Reg(Sp),
            Reg(Gp),
            Reg(Tp),
            Reg(T0),
            Reg(T1),
            Reg(T2),
            Reg(S0),
            Reg(S1),
            Reg(A0),
            Reg(A1),
            Reg(A2),
            Reg(A3),
            Reg(A4),
            Reg(A5),
            Reg(A6),
            Reg(A7),
            Reg(S2),
            Reg(S3),
            Reg(S4),
            Reg(S5),
            Reg(S6),
            Reg(S7),
            Reg(S8),
            Reg(S9),
            Reg(S10),
            Reg(S11),
            Reg(T3),
            Reg(T4),
            Reg(T5),
            Reg(T6),
            Reg(Zero),
            Reg(Ra),
            Reg(Sp),
            Reg(Gp),
            Reg(Tp),
            Reg(T0),
            Reg(T1),
            Reg(T2),
            Reg(S0),
            Reg(S1),
            Reg(A0),
            Reg(A1),
            Reg(A2),
            Reg(A3),
            Reg(A4),
            Reg(A5),
            Reg(A6),
            Reg(A7),
            Reg(S2),
            Reg(S3),
            Reg(S4),
            Reg(S5),
            Reg(S6),
            Reg(S7),
            Reg(S8),
            Reg(S9),
            Reg(S10),
            Reg(S11),
            Reg(T3),
            Reg(T4),
            Reg(T5),
            Reg(T6),
            Eof,
        ];
        for t in toks.into_iter() {
            assert_eq!(l.next_token(), t);
        }
    }

    #[test]
    fn get_opcodes() {
        use super::AsmToken::{Eof, Opcode};
        use crate::assembler::tokens::Opcode::*;
        let input = r"
        add addi and andi auipc beq bge bgeu blt bltu bne
        jal jalr lb lbu lh lhu lui lw or ori sb sh sll slli
        slt slti sltiu sltu sra srai srl srli sub sw xor xori
        ecall ebreak
        ";
        let mut l = Lexer::new(input);
        let toks = vec![
            Opcode(Add),
            Opcode(Addi),
            Opcode(And),
            Opcode(Andi),
            Opcode(Auipc),
            Opcode(Beq),
            Opcode(Bge),
            Opcode(Bgeu),
            Opcode(Blt),
            Opcode(Bltu),
            Opcode(Bne),
            Opcode(Jal),
            Opcode(Jalr),
            Opcode(Lb),
            Opcode(Lbu),
            Opcode(Lh),
            Opcode(Lhu),
            Opcode(Lui),
            Opcode(Lw),
            Opcode(Or),
            Opcode(Ori),
            Opcode(Sb),
            Opcode(Sh),
            Opcode(Sll),
            Opcode(Slli),
            Opcode(Slt),
            Opcode(Slti),
            Opcode(Sltiu),
            Opcode(Sltu),
            Opcode(Sra),
            Opcode(Srai),
            Opcode(Srl),
            Opcode(Srli),
            Opcode(Sub),
            Opcode(Sw),
            Opcode(Xor),
            Opcode(Xori),
            Opcode(Ecall),
            Opcode(Ebreak),
            Eof,
        ];

        for i in toks.into_iter() {
            assert_eq!(l.next_token(), i);
        }
    }

    #[test]
    fn get_identifier() {
        let input = r"tubias, tubias2, tubias3";
        let mut l = Lexer::new(input);
        let toks = vec![
            AsmToken::Identifier("tubias".to_string()),
            AsmToken::Comma,
            AsmToken::Identifier("tubias2".to_string()),
            AsmToken::Comma,
            AsmToken::Identifier("tubias3".to_string()),
            AsmToken::Eof,
        ];

        for t in toks.into_iter() {
            assert_eq!(l.next_token(), t);
        }
    }
}
