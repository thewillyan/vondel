use std::{iter::Peekable, rc::Rc, str::Chars};

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
        AsmToken::name_to_tok(&ident)
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
        AsmToken::Number(Rc::from(num))
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
        assert_eq!(l.next_token(), AsmToken::Label(Rc::from("tubias")));
    }

    #[test]
    fn ignore_comments() {
        let input = "# this is a comment\n tubias ; another comment here \n another_tubias #comment until end";
        let mut l = Lexer::new(input);
        assert_eq!(l.next_token(), AsmToken::Label(Rc::from("tubias")));
        assert_eq!(l.next_token(), AsmToken::Label(Rc::from("another_tubias")));
        assert_eq!(l.next_token(), AsmToken::Eof);
    }

    #[test]
    fn get_registers() {
        use super::AsmToken::{Eof, Reg};
        use crate::assembler::tokens::Register::*;
        let input = r"
        mar mdr pc mbr mbr2 cpp lv
        ra t0 t1 t2 t3
        s0 s1 s2 s3 s4 s5 s6
        a0 a1 a2 a3
        ";
        let mut l = Lexer::new(input);

        let toks = vec![
            Reg(Rc::new(Mar)),
            Reg(Rc::new(Mdr)),
            Reg(Rc::new(Pc)),
            Reg(Rc::new(Mbr)),
            Reg(Rc::new(Mbr2)),
            Reg(Rc::new(Cpp)),
            Reg(Rc::new(Lv)),
            Reg(Rc::new(Ra)),
            Reg(Rc::new(T0)),
            Reg(Rc::new(T1)),
            Reg(Rc::new(T2)),
            Reg(Rc::new(T3)),
            Reg(Rc::new(S0)),
            Reg(Rc::new(S1)),
            Reg(Rc::new(S2)),
            Reg(Rc::new(S3)),
            Reg(Rc::new(S4)),
            Reg(Rc::new(S5)),
            Reg(Rc::new(S6)),
            Reg(Rc::new(A0)),
            Reg(Rc::new(A1)),
            Reg(Rc::new(A2)),
            Reg(Rc::new(A3)),
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
        lui addi subi slti andi ori xori
        add sub slt and or sll sra sla nop not mov
        beq bne blt bge
        mul
        halt

        ";
        let mut l = Lexer::new(input);
        let toks = vec![
            Opcode(Rc::new(Lui)),
            Opcode(Rc::new(Addi)),
            Opcode(Rc::new(Subi)),
            Opcode(Rc::new(Slti)),
            Opcode(Rc::new(Andi)),
            Opcode(Rc::new(Ori)),
            Opcode(Rc::new(Xori)),
            Opcode(Rc::new(Add)),
            Opcode(Rc::new(Sub)),
            Opcode(Rc::new(Slt)),
            Opcode(Rc::new(And)),
            Opcode(Rc::new(Or)),
            Opcode(Rc::new(Sll)),
            Opcode(Rc::new(Sra)),
            Opcode(Rc::new(Sla)),
            Opcode(Rc::new(Nop)),
            Opcode(Rc::new(Not)),
            Opcode(Rc::new(Mov)),
            Opcode(Rc::new(Beq)),
            Opcode(Rc::new(Bne)),
            Opcode(Rc::new(Blt)),
            Opcode(Rc::new(Bge)),
            Opcode(Rc::new(Mul)),
            Opcode(Rc::new(Halt)),
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
            PseudoOp(Rc::new(Global)),
            PseudoOp(Rc::new(Data)),
            PseudoOp(Rc::new(Text)),
            PseudoOp(Rc::new(Word)),
            PseudoOp(Rc::new(Byte)),
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
            AsmToken::Label(Rc::from("tubias")),
            AsmToken::Comma,
            AsmToken::Label(Rc::from("tubias2")),
            AsmToken::Comma,
            AsmToken::Label(Rc::from("tubias3")),
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
            AsmToken::Number(Rc::from("1234")),
            AsmToken::Number(Rc::from("-123")),
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

        let input = r#"
add, tubias ; comment
add ra, t0 <- t1
read write"#;
        let mut l = Lexer::new(input);
        let toks = vec![
            TokWithCtx {
                tok: Rc::new(Opcode(Rc::new(Add))),
                cur_line: 2,
                cur_column: 1,
            },
            TokWithCtx {
                tok: Rc::new(Comma),
                cur_line: 2,
                cur_column: 4,
            },
            TokWithCtx {
                tok: Rc::new(Label(Rc::from("tubias"))),
                cur_line: 2,
                cur_column: 6,
            },
            TokWithCtx {
                tok: Rc::new(Opcode(Rc::new(Add))),
                cur_line: 3,
                cur_column: 1,
            },
            TokWithCtx {
                tok: Rc::new(Reg(Rc::new(Ra))),
                cur_line: 3,
                cur_column: 5,
            },
            TokWithCtx {
                tok: Rc::new(Comma),
                cur_line: 3,
                cur_column: 7,
            },
            TokWithCtx {
                tok: Rc::new(Reg(Rc::new(T0))),
                cur_line: 3,
                cur_column: 9,
            },
            TokWithCtx {
                tok: Rc::new(Assign),
                cur_line: 3,
                cur_column: 12,
            },
            TokWithCtx {
                tok: Rc::new(Reg(Rc::new(T1))),
                cur_line: 3,
                cur_column: 15,
            },
            TokWithCtx {
                tok: Rc::new(Opcode(Rc::new(Read))),
                cur_line: 4,
                cur_column: 1,
            },
            TokWithCtx {
                tok: Rc::new(Opcode(Rc::new(Write))),
                cur_line: 4,
                cur_column: 6,
            },
        ];

        for i in toks.into_iter() {
            assert_eq!(l.next_with_ctx(), i);
        }
    }
}
