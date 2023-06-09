#![allow(unused)]

pub mod alu;
pub mod mem;

use alu::Alu;
use mem::{CtrlStore, Ram, Register, Registers};

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
    state: DPState,
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
            state: DPState::default(),
            cs,
            trigger,
        }
    }

    pub fn recv_signal(&mut self, ck: &ClkLevel, mem: Ram) {
        if ck == &self.trigger {
            self.init_cycle(mem);
        } else {
            self.end_cycle(mem);
        }
    }

    pub fn init_cycle(&mut self, mem: Ram) {
        // MI (38 bits):
        // [   NEXT  |JAM|   ALU  | C BUS  |MEM| A  | B ]
        //  000000000_000_00000000_00000000_000_0000_000
        let mut curr_mi = self.cs.get_mi();

        // [ 0 | A | B ]
        self.state.enable_out = (curr_mi & 0b1111111) as u8;
        curr_mi >>= 7;

        // MEM: [ WRITE | READ | FETCH ]
        let mut mem_code = (curr_mi & 0b111) as u8;
        curr_mi >>= 3;

        let fetch = (mem_code & 1) == 1;
        if fetch {
            self.regs.mem.fetch(mem.clone());
        }
        mem_code >>= 1;

        let read = (mem_code & 1) == 1;
        if read {
            self.regs.mem.read(mem);
        }
        mem_code >>= 1;

        self.state.write = mem_code == 1;

        self.state.enable_in = (curr_mi & 0b11111111) as u8;
        curr_mi >>= 8;

        self.state.alu = (curr_mi & 0b11111111) as u8;
        curr_mi >>= 8;

        // NEXT_ADDR + JAM
        self.state.cs = curr_mi as u16;
    }

    pub fn end_cycle(&mut self, mem: Ram) {
        let b_code = self.state.enable_out & 0b111;
        let b = match b_code {
            0b000 => self.regs.mem.mdr(),
            0b001 => self.regs.sys.sp.get(),
            0b010 => self.regs.sys.lv.get(),
            0b011 => self.regs.sys.cpp.get(),
            0b100 => self.regs.gen.ob.get(),
            0b101 => self.regs.gen.sor.get(),
            _ => 0,
        };
        self.state.enable_out >>= 3;

        let a_code = self.state.enable_out & 0b1111;
        let a = match a_code {
            0b0000 => self.regs.mem.mdr(),
            0b0001 => self.regs.mem.pc(),
            0b0010 => self.regs.mem.mbr() as u32 | 0xFFFFFF00,
            0b0011 => self.regs.mem.mbr() as u32,
            0b0100 => self.regs.mem.mbr2() as u32 | 0xFFFF0000,
            0b0101 => self.regs.mem.mbr2() as u32,
            0b0111 => self.regs.sys.sp.get(),
            0b1000 => self.regs.sys.lv.get(),
            0b1001 => self.regs.sys.cpp.get(),
            0b1010 => self.regs.gen.ob.get(),
            0b1011 => self.regs.gen.sor.get(),
            _ => 0,
        };

        self.alu.entry(self.state.alu, a, b);
        let c_bus = self.alu.op();

        // | MAR | PC | SP | LV | CPP | OA | OB | SOR |
        let enb_sor_in = (self.state.enable_in & 1) == 1;
        if enb_sor_in {
            self.regs.gen.sor.set(c_bus);
        }
        self.state.enable_in >>= 1;

        let enb_ob_in = (self.state.enable_in & 1) == 1;
        if enb_ob_in {
            self.regs.gen.ob.set(c_bus);
        }
        self.state.enable_in >>= 1;

        let enb_oa_in = (self.state.enable_in & 1) == 1;
        if enb_oa_in {
            self.regs.gen.oa.set(c_bus);
        }
        self.state.enable_in >>= 1;

        let enb_cpp_in = (self.state.enable_in & 1) == 1;
        if enb_cpp_in {
            self.regs.sys.cpp.set(c_bus);
        }
        self.state.enable_in >>= 1;

        let enb_lv_in = (self.state.enable_in & 1) == 1;
        if enb_lv_in {
            self.regs.sys.lv.set(c_bus);
        }
        self.state.enable_in >>= 1;

        let enb_sp_in = (self.state.enable_in & 1) == 1;
        if enb_sp_in {
            self.regs.sys.sp.set(c_bus);
        }
        self.state.enable_in >>= 1;

        let enb_pc_in = (self.state.enable_in & 1) == 1;
        if enb_pc_in {
            self.regs.mem.update_pc(c_bus);
        }
        self.state.enable_in >>= 1;

        let enb_mar_in = (self.state.enable_in & 1) == 1;
        if enb_mar_in {
            self.regs.mem.update_mar(c_bus);
        }

        self.cs
            .update_mpc(self.state.cs, self.alu.z(), self.regs.mem.mbr());

        if self.state.write {
            self.regs.mem.write(mem);
        }
    }
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

#[derive(Debug, Default)]
struct DPState {
    cs: u16,
    alu: u8,
    enable_in: u8,
    enable_out: u8,
    write: bool,
}
