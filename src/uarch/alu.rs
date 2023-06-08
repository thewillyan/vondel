#[derive(Debug, Default)]
pub struct Alu {
    f: Func,
    a: u32,
    b: u32,
    s: Shifter,
}

impl Alu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set ALU inputs
    pub fn entry(&mut self, opcode: u8, a: u32, b: u32) {
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
        let func_code = opcode & 0b00110000;
        self.f = match func_code {
            0b00000000 => Func::And,
            0b00010000 => Func::Or,
            0b00100000 => Func::Not,
            0b00110000 => Func::Add { inc },
            _ => unreachable!(),
        };

        let (sll, sra) = match opcode & 0b11000000 {
            0b00000000 => (false, false),
            0b01000000 => (false, true),
            0b10000000 => (true, false),
            0b11000000 => (true, true),
            _ => unreachable!(),
        };
        self.s.sll = sll;
        self.s.sra = sra;
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
        };
        self.s.shift(c)
    }
}

#[derive(Debug)]
enum Func {
    And,
    Or,
    Not,
    Add { inc: bool },
}

impl Default for Func {
    fn default() -> Self {
        Self::And
    }
}

#[derive(Debug, Default)]
struct Shifter {
    sll: bool,
    sra: bool,
}

impl Shifter {
    fn shift(&self, mut entry: u32) -> u32 {
        if self.sra {
            entry >>= 1;
        }
        if self.sll {
            entry <<= 8;
        }
        entry
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
        alu.entry(0b10111101, A, B);
        assert_eq!((A + B + 1) << 8, alu.op());
    }

    #[test]
    fn sra() {
        let mut alu = Alu::default();
        alu.entry(0b01111101, A, B);
        assert_eq!((A + B + 1) >> 1, alu.op());
    }

    #[test]
    fn sra_and_sll() {
        let mut alu = Alu::default();
        alu.entry(0b11111101, A, B);
        assert_eq!((A + B + 1) >> 1 << 8, alu.op());
    }
}
