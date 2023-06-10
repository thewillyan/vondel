use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub mod alu;
pub mod mem;

use alu::Alu;
use mem::{CtrlStore, Ram, Register, Registers};

#[derive(Debug)]
pub struct Computer {
    mem: Ram,
    cpu: Cpu,
    clock: Arc<Mutex<Clock>>,
}

impl Computer {
    pub fn new<M, F>(mem: M, firmware: F) -> Self
    where
        M: IntoIterator<Item = u32>,
        F: IntoIterator<Item = u64>,
    {
        let mut ram = Ram::default();
        ram.load(0, mem);
        let cs = CtrlStore::builder().load(0, firmware).build();
        Self {
            mem: ram,
            cpu: Cpu::new(cs),
            clock: Arc::new(Mutex::new(Clock::default())),
        }
    }

    pub fn exec(&mut self) {
        let (tx, rx) = mpsc::sync_channel(1);
        let clk = Arc::clone(&self.clock);
        thread::spawn(move || loop {
            let mut clk = clk.lock().expect("Cannot get the clock lock.");
            tx.send(clk.lv.clone())
                .expect("Failed to send clock signal");
            clk.alt();
        });
        self.cpu.run(self.mem.clone(), rx);
    }
}

#[derive(Debug)]
struct Cpu {
    thr: Thread,
    firmware: CtrlStore,
}

impl Cpu {
    pub fn new(firmware: CtrlStore) -> Self {
        Self {
            thr: Thread::new(),
            firmware,
        }
    }

    pub fn run(&mut self, mem: Ram, recver: mpsc::Receiver<ClkLevel>) {
        self.thr.init(self.firmware.get_mi(), mem.clone());
        while let Ok(trigger) = recver.recv() {
            if self.firmware.get_mi() != CtrlStore::TERM {
                break;
            }
            self.thr.step(&trigger, mem.clone(), self.firmware.clone());
        }
    }
}

#[derive(Debug)]
struct Thread {
    dp1: DataPath,
    dp2: DataPath,
}

impl Thread {
    pub fn new() -> Self {
        Self::default()
    }

    fn init(&mut self, mi: u64, mem: Ram) {
        self.dp1.init_cycle(mi, mem);
    }

    fn step(&mut self, trigger: &ClkLevel, mem: Ram, cs: CtrlStore) {
        if trigger == &self.dp1.trigger {
            self.dp2.end_cycle(mem.clone(), cs.clone());
            self.dp1.init_cycle(cs.get_mi(), mem);
        } else {
            self.dp1.end_cycle(mem.clone(), cs.clone());
            self.dp2.init_cycle(cs.get_mi(), mem);
        }
    }
}

impl Default for Thread {
    fn default() -> Self {
        let trigger1 = ClkLevel::default();
        let trigger2 = trigger1.inv();
        Thread {
            dp1: DataPath::new(trigger1),
            dp2: DataPath::new(trigger2),
        }
    }
}

#[derive(Debug)]
pub struct DataPath {
    alu: Alu,
    regs: Registers,
    state: DPState,
    trigger: ClkLevel,
}

impl DataPath {
    pub fn new(trigger: ClkLevel) -> Self {
        Self {
            alu: Alu::new(),
            regs: Registers::new(),
            state: DPState::default(),
            trigger,
        }
    }

    pub fn recv_signal(&mut self, ck: &ClkLevel, mem: Ram, cs: CtrlStore) {
        if ck == &self.trigger {
            self.init_cycle(cs.get_mi(), mem);
        } else {
            self.end_cycle(mem, cs);
        }
    }

    pub fn init_cycle(&mut self, mut mi: u64, mem: Ram) {
        // [ 0 | A | B ]
        self.state.enable_out = (mi & 0b1111111) as u8;
        mi >>= 7;

        // MEM: [ WRITE | READ | FETCH ]
        let mut mem_code = (mi & 0b111) as u8;
        mi >>= 3;

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

        self.state.enable_in = (mi & 0b11111111) as u8;
        mi >>= 8;

        self.state.alu = (mi & 0b11111111) as u8;
        mi >>= 8;

        // NEXT_ADDR + JAM
        self.state.cs = mi as u16;
    }

    pub fn end_cycle(&mut self, mem: Ram, cs: CtrlStore) {
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
            self.regs.mem.update_pc(c_bus, mem.clone());
        }
        self.state.enable_in >>= 1;

        let enb_mar_in = (self.state.enable_in & 1) == 1;
        if enb_mar_in {
            self.regs.mem.update_mar(c_bus);
        }

        if self.state.write {
            self.regs.mem.write(mem);
        }

        cs.update_mpc(self.state.cs, self.alu.z(), self.regs.mem.mbr());
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
        self.lv = self.lv.inv();
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ClkLevel {
    Falling,
    Rising,
}

impl Default for ClkLevel {
    fn default() -> Self {
        Self::Falling
    }
}

impl ClkLevel {
    pub fn inv(&self) -> Self {
        match self {
            ClkLevel::Falling => ClkLevel::Rising,
            ClkLevel::Rising => ClkLevel::Falling,
        }
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
