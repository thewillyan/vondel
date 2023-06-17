use std::{iter::Peekable, str::Chars};

use crate::assembler::tokens::{AsmToken, TokWithCtx};

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
                        self.cur_column = 0;
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

    fn read_number(&mut self) -> AsmToken {
        let mut num = String::from(self.cur_char);
        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() {
                num.push(c);
                self.read_char();
            } else {
                break;
            }
        }
        AsmToken::Number(num)
    }

    fn tokenizer(&mut self) -> AsmToken {
        match self.cur_char {
            ':' => AsmToken::Colon,
            ',' => AsmToken::Comma,
            '<' => match self.chars.peek() {
                Some(&'-') => {
                    self.read_char();
                    AsmToken::Assign
                }
                _ => AsmToken::Illegal,
            },
            c if c.is_alphabetic() || c == '.' || c == '_' => self.read_identifier(),
            c if c == '-' || c.is_numeric() => self.read_number(),
            '\0' => AsmToken::Eof,
            _ => AsmToken::Illegal,
        }
    }

    fn next_token(&mut self) -> AsmToken {
        self.skip_whitespace();
        self.ignore_comment();
        let tok = self.tokenizer();
        self.read_char();
        tok
    }

    pub fn next_with_ctx(&mut self) -> TokWithCtx {
        self.skip_whitespace();
        self.ignore_comment();
        let cur_line = self.cur_line;
        let cur_column = self.cur_column;
        let tok = self.tokenizer();
        let tok_ctx = TokWithCtx::new(tok, cur_line, cur_column);
        self.read_char();

        tok_ctx
    }

    pub fn get_deez_toks(&mut self) -> Vec<AsmToken> {
        let mut toks = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == AsmToken::Eof {
                toks.push(tok);
                break;
            }
            toks.push(tok);
        }
        toks
    }

    pub fn get_deez_toks_w_ctx(&mut self) -> Vec<TokWithCtx> {
        let mut toks = Vec::new();
        loop {
            let tok = self.next_with_ctx();
            if *tok.tok == AsmToken::Eof {
                toks.push(tok);
                break;
            }
            toks.push(tok);
        }
        toks
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    #[test]
    fn skip_whitespace() {
        let input = "    \t\n tubias";
        let mut l = Lexer::new(input);
        assert_eq!(l.next_token(), AsmToken::Label("tubias".to_string()));
    }

    #[test]
    fn ignore_comments() {
        let input = "# this is a comment\n tubias ; another comment here \n another_tubias #comment until end";
        let mut l = Lexer::new(input);
        assert_eq!(l.next_token(), AsmToken::Label("tubias".to_string()));
        assert_eq!(
            l.next_token(),
            AsmToken::Label("another_tubias".to_string())
        );
        assert_eq!(l.next_token(), AsmToken::Eof);
    }

    #[test]
    fn get_registers() {
        use super::AsmToken::{Eof, Reg};
        use crate::assembler::tokens::Register::*;
        let input = r"
        ra sp cpp lv
        t0 t1 t2 t3
        s0 s1 s2 s3 s4 s5 s6
        a0 a1 a2 a3

        x0 x1 x2 x3
        x4 x5 x6 x7
        x8 x9 x10 x11 x12 x13 x14
        x15 x16 x17 x18
        ";
        let mut l = Lexer::new(input);

        let toks = vec![
            Reg(Ra),
            Reg(Sp),
            Reg(Cpp),
            Reg(Lv),
            Reg(T0),
            Reg(T1),
            Reg(T2),
            Reg(T3),
            Reg(S0),
            Reg(S1),
            Reg(S2),
            Reg(S3),
            Reg(S4),
            Reg(S5),
            Reg(S6),
            Reg(A0),
            Reg(A1),
            Reg(A2),
            Reg(A3),
            Reg(Ra),
            Reg(Sp),
            Reg(Cpp),
            Reg(Lv),
            Reg(T0),
            Reg(T1),
            Reg(T2),
            Reg(T3),
            Reg(S0),
            Reg(S1),
            Reg(S2),
            Reg(S3),
            Reg(S4),
            Reg(S5),
            Reg(S6),
            Reg(A0),
            Reg(A1),
            Reg(A2),
            Reg(A3),
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
        ;addi slti andi ori xori slli srli lui auipc\n
        add sub slt and or sll srl nop
        beq bne blt bge
        mul
        halt

        ";
        let mut l = Lexer::new(input);
        let toks = vec![
            // Opcode(Addi),
            // Opcode(Slti),
            // Opcode(Andi),
            // Opcode(Ori),
            // Opcode(Xori),
            // Opcode(Slli),
            // Opcode(Srli),
            // Opcode(Lui),
            // Opcode(Auipc),
            Opcode(Add),
            Opcode(Sub),
            Opcode(Slt),
            Opcode(And),
            Opcode(Or),
            Opcode(Sll),
            Opcode(Srl),
            Opcode(Nop),
            Opcode(Beq),
            Opcode(Bne),
            Opcode(Blt),
            Opcode(Bge),
            Opcode(Mul),
            Opcode(Halt),
            Eof,
        ];

        for i in toks.into_iter() {
            assert_eq!(l.next_token(), i);
        }
    }

    #[test]
    fn get_pseudoinstructions() {
        use super::AsmToken::{Eof, PseudoIns};
        use crate::assembler::tokens::PseudoInstruction::*;
        let input = r"
        mv neg seqz snez sltz sgtz beqz bnez blez bgez bltz bgtz bgt ble
        ";
        let mut l = Lexer::new(input);
        let toks = vec![
            PseudoIns(Mv),
            PseudoIns(Neg),
            PseudoIns(Seqz),
            PseudoIns(Snez),
            PseudoIns(Sltz),
            PseudoIns(Sgtz),
            PseudoIns(Beqz),
            PseudoIns(Bnez),
            PseudoIns(Blez),
            PseudoIns(Bgez),
            PseudoIns(Bltz),
            PseudoIns(Bgtz),
            PseudoIns(Bgt),
            PseudoIns(Ble),
            Eof,
        ];

        for i in toks.into_iter() {
            assert_eq!(l.next_token(), i);
        }
    }

    #[test]
    fn get_pseudo_ops() {
        use super::AsmToken::{Eof, Illegal, PseudoOp};
        use crate::assembler::tokens::PseudoOps::*;
        let input = r"
        .global .data .text .word .byte .tubias
        ";
        let mut l = Lexer::new(input);
        let toks = vec![
            PseudoOp(Global),
            PseudoOp(Data),
            PseudoOp(Text),
            PseudoOp(Word),
            PseudoOp(Byte),
            Illegal,
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
            AsmToken::Label("tubias".to_string()),
            AsmToken::Comma,
            AsmToken::Label("tubias2".to_string()),
            AsmToken::Comma,
            AsmToken::Label("tubias3".to_string()),
            AsmToken::Eof,
        ];

        for t in toks.into_iter() {
            assert_eq!(l.next_token(), t);
        }
    }

    #[test]
    fn get_numbers() {
        let input = "1234 -123";
        let mut l = Lexer::new(input);
        let toks = vec![
            AsmToken::Number(String::from("1234")),
            AsmToken::Number(String::from("-123")),
            AsmToken::Eof,
        ];

        for t in toks.into_iter() {
            assert_eq!(l.next_token(), t);
        }
    }

    #[test]
    fn get_toks_with_ctx() {
        use super::AsmToken::*;
        use crate::assembler::tokens::Opcode::*;
        use crate::assembler::tokens::Register::*;

        let input = "add, tubias ; comment\n add ra, t0 <- t1\n read write";
        let mut l = Lexer::new(input);
        let toks = vec![
            TokWithCtx {
                tok: Rc::new(Opcode(Add)),
                cur_line: 1,
                cur_column: 1,
            },
            TokWithCtx {
                tok: Rc::new(Comma),
                cur_line: 1,
                cur_column: 4,
            },
            TokWithCtx {
                tok: Rc::new(Label("tubias".to_string())),
                cur_line: 1,
                cur_column: 6,
            },
            TokWithCtx {
                tok: Rc::new(Opcode(Add)),
                cur_line: 2,
                cur_column: 2,
            },
            TokWithCtx {
                tok: Rc::new(Reg(Ra)),
                cur_line: 2,
                cur_column: 6,
            },
            TokWithCtx {
                tok: Rc::new(Comma),
                cur_line: 2,
                cur_column: 8,
            },
            TokWithCtx {
                tok: Rc::new(Reg(T0)),
                cur_line: 2,
                cur_column: 10,
            },
            TokWithCtx {
                tok: Rc::new(Assign),
                cur_line: 2,
                cur_column: 13,
            },
            TokWithCtx {
                tok: Rc::new(Reg(T1)),
                cur_line: 2,
                cur_column: 16,
            },
            TokWithCtx {
                tok: Rc::new(Opcode(Read)),
                cur_line: 3,
                cur_column: 2,
            },
            TokWithCtx {
                tok: Rc::new(Opcode(Write)),
                cur_line: 3,
                cur_column: 7,
            },
        ];

        for i in toks.into_iter() {
            assert_eq!(l.next_with_ctx(), i);
        }
    }
}
