pub mod alu;
pub mod mem;

#[derive(Debug)]
struct Cpu {}

#[derive(Debug, Default)]
struct Clock {
    lv: ClkLevel,
    count: u32,
}

impl Clock {
    fn alt(&mut self) {
        self.count += 1;
        self.lv = match self.lv {
            ClkLevel::Falling => ClkLevel::Rising,
            ClkLevel::Rising => ClkLevel::Falling,
        }
    }
}

#[derive(Debug)]
enum ClkLevel {
    Falling,
    Rising,
}

impl Default for ClkLevel {
    fn default() -> Self {
        Self::Falling
    }
}
