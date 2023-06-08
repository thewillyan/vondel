pub mod alu;
pub mod mem;

use alu::Alu;
use mem::{CtrlStore, Ram, Registers};

#[derive(Debug)]
struct Computer {
    mem: Ram,
    cpu: Cpu,
    clock: Clock,
}

#[derive(Debug)]
struct Cpu {
    thr: Thread,
}

#[derive(Debug)]
struct Thread {
    dp1: DataPath,
    dp2: DataPath,
}

#[derive(Debug)]
pub struct DataPath {
    alu: Alu,
    regs: Registers,
    cs: CtrlStore,
    trigger: ClkLevel,
}

impl DataPath {
    pub fn new<T>(firmware: T, trigger: ClkLevel) -> Self
    where
        T: IntoIterator<Item = u64>,
    {
        let cs = CtrlStore::builder().load(0, firmware).build();
        Self {
            alu: Alu::new(),
            regs: Registers::new(),
            cs,
            trigger,
        }
    }

    pub fn recv_signal(&self, ck: &ClkLevel) {
        if ck == &self.trigger {
            self.init_cycle();
        } else {
            self.end_cycle();
        }
    }

    pub fn init_cycle(&self) {}

    pub fn end_cycle(&self) {}
}

#[derive(Debug, Default)]
pub struct Clock {
    lv: ClkLevel,
    count: u32,
}

impl Clock {
    pub fn alt(&mut self) {
        self.count += 1;
        self.lv = match self.lv {
            ClkLevel::Falling => ClkLevel::Rising,
            ClkLevel::Rising => ClkLevel::Falling,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClkLevel {
    Falling,
    Rising,
}

impl Default for ClkLevel {
    fn default() -> Self {
        Self::Falling
    }
}
