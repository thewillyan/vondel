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
    pub fn new(mem: Ram, firmware: CtrlStore) -> Self {
        Self {
            mem,
            cpu: Cpu::new(firmware),
            clock: Arc::new(Mutex::new(Clock::default())),
        }
    }

    pub fn exec(&mut self) {
        self.cpu.thr_sync();
        self.clock.lock().expect("Cannot get the clock lock.").alt();

        let (tx, rx) = mpsc::sync_channel(0);
        let clk = Arc::clone(&self.clock);

        thread::spawn(move || loop {
            let mut clk = clk.lock().expect("Cannot get the clock lock.");
            match tx.send(clk.lv.clone()) {
                Ok(_) => clk.alt(),
                Err(_) => break,
            }
        });
        self.cpu.run(&mut self.mem, rx);
    }

    pub fn cycles(&self) -> f64 {
        let alts = self.clock.lock().expect("Cannot get the clock lock.").count as f64;
        alts / 2.0
    }

    pub fn regs(&self) -> &Registers {
        self.cpu.thr.regs()
    }
}

#[derive(Debug)]
struct Cpu {
    pub thr: Thread,
    firmware: CtrlStore,
}

impl Cpu {
    pub fn new(firmware: CtrlStore) -> Self {
        Self {
            thr: Thread::new(),
            firmware,
        }
    }

    pub fn thr_sync(&mut self) {
        self.thr.sync(&self.firmware);
    }

    pub fn run(&mut self, mem: &mut Ram, recver: mpsc::Receiver<ClkLevel>) {
        for trigger in recver {
            if self.firmware.get_mi() == CtrlStore::TERM {
                break;
            }
            self.thr.step(&trigger, mem, &self.firmware);
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

    fn sync(&mut self, cs: &CtrlStore) {
        self.dp1.init_cycle(cs);
    }

    fn step(&mut self, trigger: &ClkLevel, mem: &mut Ram, cs: &CtrlStore) {
        if trigger == &self.dp1.trigger {
            self.dp2.end_cycle(mem, cs);
            self.dp1.init_cycle(cs);
        } else {
            self.dp1.end_cycle(mem, cs);
            self.dp2.init_cycle(cs);
        }
    }

    pub fn regs(&self) -> &Registers {
        &self.dp1.regs
    }
}

impl Default for Thread {
    fn default() -> Self {
        let dp1 = DataPath::new(ClkLevel::default());
        let dp2 = dp1.sibling();
        Thread { dp1, dp2 }
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

    pub fn sibling(&self) -> Self {
        DataPath {
            alu: Alu::default(),
            regs: Registers::from(&self.regs),
            state: DPState::default(),
            trigger: self.trigger.inv(),
        }
    }

    pub fn recv_signal(&mut self, ck: &ClkLevel, mem: &mut Ram, cs: &CtrlStore) {
        if ck == &self.trigger {
            self.init_cycle(cs);
        } else {
            self.end_cycle(mem, cs);
        }
    }

    pub fn init_cycle(&mut self, cs: &CtrlStore) {
        let mut mi = cs.get_mi();
        // [ 0 | A | B ]
        let mut enable_out = (mi & 0b1111111) as u8;
        let b_code = enable_out & 0b111;
        self.state.b = match b_code {
            0b000 => self.regs.mem.mdr(),
            0b001 => self.regs.sys.sp.get(),
            0b010 => self.regs.sys.lv.get(),
            0b011 => self.regs.sys.cpp.get(),
            0b100 => self.regs.gen.ob.get(),
            0b101 => self.regs.gen.sor.get(),
            _ => 0,
        };
        enable_out >>= 3;

        let a_code = enable_out & 0b1111;
        self.state.a = match a_code {
            0b0000 => self.regs.mem.mdr(),
            0b0001 => self.regs.mem.pc(),
            0b0010 => self.regs.mem.mbr() as u32 | 0xFFFFFF00,
            0b0011 => self.regs.mem.mbr() as u32,
            0b0100 => self.regs.mem.mbr2() as u32 | 0xFFFF0000,
            0b0101 => self.regs.mem.mbr2() as u32,
            0b0110 => self.regs.sys.sp.get(),
            0b0111 => self.regs.sys.lv.get(),
            0b1000 => self.regs.sys.cpp.get(),
            0b1001 => self.regs.gen.oa.get(),
            0b1010 => self.regs.gen.sor.get(),
            _ => 0,
        };
        mi >>= 7;

        // MEM: [ WRITE | READ | FETCH ]
        let mut mem_code = (mi & 0b111) as u8;
        mi >>= 3;
        self.state.fetch = (mem_code & 1) == 1;
        mem_code >>= 1;
        self.state.read = (mem_code & 1) == 1;
        mem_code >>= 1;
        self.state.write = mem_code == 1;

        self.state.enable_in = (mi & 0b11111111) as u8;
        mi >>= 8;
        self.state.alu_entry = (mi & 0b11111111) as u8;
        mi >>= 8;

        // NEXT_ADDR + JAM
        self.state.cs_opcode = mi as u16;
    }

    pub fn end_cycle(&mut self, mem: &mut Ram, cs: &CtrlStore) {
        self.alu
            .entry(self.state.alu_entry, self.state.a, self.state.b);
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

        cs.update_mpc(self.state.cs_opcode, self.alu.z(), &mut self.regs.mem);

        // MEMORY
        if self.state.read {
            self.regs.mem.read(mem);
        }
        if self.state.write {
            self.regs.mem.write(mem);
        }
        if self.state.fetch {
            self.regs.mem.fetch(mem);
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
    cs_opcode: u16,
    alu_entry: u8,
    enable_in: u8,
    a: u32,
    b: u32,
    write: bool,
    read: bool,
    fetch: bool,
}
