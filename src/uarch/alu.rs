use crate::uarch::mem::{Reg, Register};

#[derive(Debug, Default)]
pub struct Alu {
    f: Func,
    a: u32,
    b: u32,
    s: Shifter,
    z: Reg<bool>,
    n: Reg<bool>,
}

impl Alu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set ALU inputs
    pub fn entry(&mut self, opcode: u16, a: u32, b: u32) {
        let inc = (opcode & 0b00000001) == 0b00000001;
        let inva = (opcode & 0b00000010) == 0b00000010;

        let enable = opcode & 0b00001100;
        let (a, b) = match enable {
            0b00000000 => (0, 0),
            0b00000100 => (0, b),
            0b00001000 => (a, 0),
            0b00001100 => (a, b),
            _ => unreachable!(),
        };
        let a = if inva { !a } else { a };
        (self.a, self.b) = (a, b);

        // get f0 and f1
        let func_code = opcode & 0b001110000;
        self.f = match func_code {
            0b00000000 => Func::And,
            0b00010000 => Func::Or,
            0b00100000 => Func::Not,
            0b00110000 => Func::Add { inc },
            0b01000000 => Func::Xor,
            0b01010000 => Func::Mul,
            0b01100000 => Func::Div,
            0b01110000 => Func::Mod,
            _ => unreachable!(),
        };
        self.s.entry = (opcode >> 7) as u8;
    }

    pub fn op(&self) -> u32 {
        let c = match &self.f {
            Func::Add { inc } => {
                let mut sum = self.a as i32 + self.b as i32;
                if *inc {
                    sum += 1;
                }
                sum as u32
            }
            Func::And => self.a & self.b,
            Func::Or => self.a | self.b,
            Func::Not => !self.b,
            Func::Xor => self.a ^ self.b,
            Func::Mul => self.a * self.b,
            Func::Div => self.a / self.b,
            Func::Mod => self.a % self.b,
        };
        self.z.set(c == 0);
        self.n.set((c >> 31) == 1);
        self.s.shift(c)
    }

    pub fn z(&self) -> bool {
        self.z.get()
    }

    pub fn n(&self) -> bool {
        self.n.get()
    }
}

#[derive(Debug)]
enum Func {
    And,
    Or,
    Xor,
    Not,
    Add { inc: bool },
    Mul,
    Div,
    Mod,
}

impl Default for Func {
    fn default() -> Self {
        Self::And
    }
}

#[derive(Debug, Default)]
struct Shifter {
    entry: u8,
}

impl Shifter {
    fn shift(&self, input: u32) -> u32 {
        match self.entry {
            0b00 => input,
            0b01 => input >> 1,
            0b10 => input << 8,
            0b11 => input << 1,
            _ => unreachable!("The shifter entry must have only 2 bits"),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    const A: u32 = 8;
    const B: u32 = 42;

    #[test]
    fn and() {
        let mut alu = Alu::default();
        alu.entry(0b00001100, A, B);
        assert_eq!(A & B, alu.op());
    }

    #[test]
    fn or() {
        let mut alu = Alu::default();
        alu.entry(0b00011000, A, B);
        assert_eq!(A, alu.op());

        alu.entry(0b00010100, A, B);
        assert_eq!(B, alu.op());

        alu.entry(0b00011010, A, B);
        assert_eq!(!A, alu.op());

        alu.entry(0b00011100, A, B);
        assert_eq!(A | B, alu.op());

        alu.entry(0b00010000, A, B);
        assert_eq!(0, alu.op());
    }

    #[test]
    fn xor() {
        let mut alu = Alu::default();
        alu.entry(0b001001100, A, B);
        assert_eq!(A ^ B, alu.op());
    }

    #[test]
    fn mul() {
        let mut alu = Alu::default();
        alu.entry(0b001011100, A, B);
        assert_eq!(A * B, alu.op());
    }

    #[test]
    fn div() {
        let mut alu = Alu::default();
        alu.entry(0b001101100, A, B);
        assert_eq!(A / B, alu.op());
    }

    #[test]
    fn div_mod() {
        let mut alu = Alu::default();
        alu.entry(0b001111100, A, B);
        assert_eq!(A % B, alu.op());
    }

    #[test]
    fn not() {
        let mut alu = Alu::default();
        alu.entry(0b00101100, A, B);
        assert_eq!(!B, alu.op());
    }

    #[test]
    fn add() {
        let mut alu = Alu::default();
        alu.entry(0b00111100, A, B);
        assert_eq!(A + B, alu.op());

        alu.entry(0b00111101, A, B);
        assert_eq!(A + B + 1, alu.op());

        alu.entry(0b00111001, A, B);
        assert_eq!(A + 1, alu.op());

        alu.entry(0b00110101, A, B);
        assert_eq!(B + 1, alu.op());

        alu.entry(0b00111111, A, B);
        assert_eq!(B - A, alu.op());

        alu.entry(0b00110110, A, B);
        assert_eq!(B - 1, alu.op());

        alu.entry(0b00111011, A, B);
        assert_eq!(-(A as i32), alu.op() as i32);

        alu.entry(0b00110001, A, B);
        assert_eq!(1, alu.op());

        alu.entry(0b00110010, A, B);
        assert_eq!(-1i32, alu.op() as i32);
    }

    #[test]
    fn sll() {
        let mut alu = Alu::default();
        alu.entry(0b100111101, A, B);
        assert_eq!((A + B + 1) << 8, alu.op());
    }

    #[test]
    fn sla() {
        let mut alu = Alu::default();
        alu.entry(0b110111101, A, B);
        assert_eq!((A + B + 1) << 1, alu.op());
    }

    #[test]
    fn sra() {
        let mut alu = Alu::default();
        alu.entry(0b010111101, A, B);
        assert_eq!((A + B + 1) >> 1, alu.op());
    }

    #[test]
    fn is_zero() {
        let mut alu = Alu::default();
        alu.entry(0b00111111, A, A);
        assert_eq!(0, alu.op());
        assert!(alu.z());
    }

    #[test]
    fn is_neg() {
        let mut alu = Alu::default();
        alu.entry(0b00111111, B, A);
        assert_eq!(A as i32 - B as i32, alu.op() as i32);
        assert!(alu.n());
    }
}
