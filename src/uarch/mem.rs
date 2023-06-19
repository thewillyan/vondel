use std::{
    cell::Cell,
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Mutex},
};

// 32-bit word * 20-bit addr = about 4 MB
const RAM_ADDRS: usize = 2usize.pow(20);
// 9-bit addr
const CS_ADDRS: usize = 2usize.pow(9);

#[derive(Debug)]
pub struct Ram {
    data: Arc<Mutex<Box<[u32; RAM_ADDRS]>>>,
}

impl Ram {
    pub fn new() -> Self {
        Self::default()
    }

    // get the nth word of the memory
    pub fn get(&self, n: u32) -> u32 {
        self.data.lock().expect("Failed to get the RAM lock")[n as usize]
    }

    // set the nth word of the memory to `v`
    pub fn set(&mut self, n: u32, v: u32) {
        let mut words = self.data.lock().expect("Failed to get the RAM lock");
        words[n as usize] = v
    }

    /// load words from `v` starting at the nth memory word
    pub fn load<T: IntoIterator<Item = u32>>(&mut self, n: u32, v: T) {
        for (i, word) in v.into_iter().enumerate() {
            self.set(i as u32 + n, word);
        }
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(Box::new([0; RAM_ADDRS]))),
        }
    }
}

impl Clone for Ram {
    fn clone(&self) -> Self {
        Ram {
            data: Arc::clone(&self.data),
        }
    }
}

#[derive(Debug)]
pub struct CtrlStoreBuilder {
    firmware: [u64; CS_ADDRS],
    mpc: u16,
}

impl CtrlStoreBuilder {
    /// set the nth word of the memory to `v`
    pub fn set(mut self, n: u16, v: u64) -> Self {
        self.firmware[n as usize] = v;
        self
    }

    /// load the microintructions of `v` starting at the nth memory word
    pub fn load<T: IntoIterator<Item = u64>>(mut self, n: u16, v: T) -> Self {
        for (i, mi) in v.into_iter().enumerate() {
            self.firmware[i + n as usize] = mi;
        }
        self
    }

    pub fn set_mpc(mut self, byte: u16) -> Self {
        self.mpc = byte & 0b0000000111111111;
        self
    }

    /// Build a `CtrlStore`
    pub fn build(self) -> CtrlStore {
        CtrlStore {
            firmware: Arc::new(self.firmware),
            mpc: SharedReg::new(self.mpc),
        }
    }
}

impl Default for CtrlStoreBuilder {
    fn default() -> Self {
        CtrlStoreBuilder {
            firmware: [0; CS_ADDRS],
            mpc: 0,
        }
    }
}

#[derive(Debug)]
pub struct CtrlStore {
    firmware: Arc<[u64; CS_ADDRS]>,
    mpc: SharedReg<u16>,
}

impl CtrlStore {
    /// The mucroinstruction that indicates that the program must stop.
    pub const HALT: u64 = u64::MAX;

    pub fn builder() -> CtrlStoreBuilder {
        CtrlStoreBuilder::default()
    }

    /// Get the next Microinstruction from the CtrlStore, in other words,
    /// fetch the MI at the position stored at MPC.
    ///
    /// A single microintruction is formated as shown in
    /// [this diagram](https://i.imgur.com/tlHAPgL.png).
    pub fn get_mi(&self) -> u64 {
        self.firmware[self.mpc.get() as usize]
    }

    /// Update the MPC from the opcode of the format:
    ///
    /// `[ ... | NEXT_ADDR | JMPC | JAMN | JAMZ ]`
    ///
    /// where `JMPC`, `JAMN` and `JAMZ` are 1-bit wide and `NEXT_ADDR` is
    /// 9-bit wide. The 4 bits represented by `...` are ignored.
    pub fn update_mpc(&self, mut opcode: u16, z: bool, n: bool, mem_regs: &mut MemRegs) {
        // ignored 4 MSBs
        opcode &= 0b0000111111111111;

        let jamz = (opcode & 1) == 1;
        opcode >>= 1;
        let jamn = (opcode & 1) == 1;
        opcode >>= 1;
        let jmpc = (opcode & 1) == 1;
        opcode >>= 1;

        let mut next_addr = opcode;

        if jamn && n {
            next_addr |= 0b0000000100000000;
        }

        if jamz && z {
            next_addr |= 0b0000000100000000;
        }

        if jmpc {
            next_addr |= mem_regs.mbr() as u16;
        }

        self.mpc.set(next_addr);
    }
}

impl Clone for CtrlStore {
    fn clone(&self) -> Self {
        CtrlStore {
            firmware: Arc::clone(&self.firmware),
            mpc: self.mpc.clone(),
        }
    }
}

pub trait Register {
    type Item;
    fn get(&self) -> Self::Item;
    fn set(&self, v: Self::Item);
}

#[derive(Debug, Clone)]
pub struct Reg<T: Copy> {
    v: Rc<Cell<T>>,
}

impl<T: Copy> Reg<T> {
    pub fn new(v: T) -> Self {
        Self {
            v: Rc::new(Cell::new(v)),
        }
    }
}

impl<T: Copy + Default> Default for Reg<T> {
    fn default() -> Self {
        let v = Rc::new(Cell::new(T::default()));
        Self { v }
    }
}

impl<T: Copy> Register for Reg<T> {
    type Item = T;

    fn get(&self) -> Self::Item {
        self.v.get()
    }

    fn set(&self, v: Self::Item) {
        self.v.set(v);
    }
}

#[derive(Debug)]
pub struct SharedReg<T> {
    v: Arc<Mutex<T>>,
}

impl<T> SharedReg<T> {
    pub fn new(v: T) -> Self {
        Self {
            v: Arc::new(Mutex::new(v)),
        }
    }
}

impl<T: Default> Default for SharedReg<T> {
    fn default() -> Self {
        let v = Arc::new(Mutex::new(T::default()));
        Self { v }
    }
}

impl<T> Clone for SharedReg<T> {
    fn clone(&self) -> Self {
        Self {
            v: Arc::clone(&self.v),
        }
    }
}

impl<T: Copy> Register for SharedReg<T> {
    type Item = T;

    fn get(&self) -> Self::Item {
        *self
            .v
            .lock()
            .expect("Failed to get lock on shared register")
    }

    fn set(&self, v: Self::Item) {
        let mut lock = self
            .v
            .lock()
            .expect("Failed to get lock on shared register");
        *lock = v
    }
}

#[derive(Debug, Default)]
pub struct MemRegs {
    mar: SharedReg<u32>,
    mdr: SharedReg<u32>,
    pc: SharedReg<u32>,
    mbr: SharedReg<u8>,
    mbr2: SharedReg<u16>,
    ifu: Arc<Mutex<Ifu>>,
}

impl MemRegs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mar(&self) -> u32 {
        self.mar.get()
    }

    pub fn mdr(&self) -> u32 {
        self.mdr.get()
    }

    pub fn pc(&self) -> u32 {
        self.pc.get()
    }

    pub fn mbr(&mut self) -> u8 {
        let val = self.mbr.get();
        let mut ifu = self.ifu.lock().expect("failed to get the IFU lock");
        ifu.consume_mbr();
        ifu.load(&self.mbr, &self.mbr2);
        self.pc.set(self.pc.get() + 2);
        val
    }

    pub fn mbr2(&mut self) -> u16 {
        let val = self.mbr2.get();
        let mut ifu = self.ifu.lock().expect("failed to get the IFU lock");
        ifu.consume_mbr2();
        ifu.load(&self.mbr, &self.mbr2);
        self.pc.set(self.pc.get() + 2);
        val
    }

    pub fn read(&mut self, mem: &Ram) {
        self.mdr.set(mem.get(self.mar.get()));
    }

    pub fn write(&mut self, mem: &mut Ram) {
        mem.set(self.mar.get(), self.mdr.get())
    }

    pub fn fetch(&mut self, mem: &Ram) {
        let mut ifu = self.ifu.lock().expect("failed to get the IFU lock");
        ifu.fetch(mem);
        ifu.load(&self.mbr, &self.mbr2);
    }

    pub fn update_pc(&mut self, v: u32) {
        self.pc.set(v);
        let mut ifu_lock = self.ifu.lock().expect("failed to get the IFU lock");
        ifu_lock.imar = v;
    }

    pub fn update_mar(&mut self, v: u32) {
        self.mar.set(v)
    }

    pub fn update_mdr(&mut self, v: u32) {
        self.mdr.set(v)
    }
}

/// A Instruction Fetch Unit with 8 bytes of cache
#[derive(Debug)]
struct Ifu {
    cache: VecDeque<u8>,
    imar: u32,
}

impl Ifu {
    /// fetches a word (4 bytes) from the memory if has capacity, the max capacity is 7 bytes.
    fn fetch(&mut self, mem: &Ram) {
        if self.cache.len() < 4 {
            let word = mem.get(self.imar);
            self.imar += 1;
            for b in word.to_le_bytes() {
                self.cache.push_back(b);
            }
        }
    }

    fn load(&mut self, mbr: &SharedReg<u8>, mbr2: &SharedReg<u16>) {
        let a = self.cache.front().copied().unwrap_or(0);
        let b = self.cache.get(1).copied().unwrap_or(0);
        mbr.set(a);
        mbr2.set((b as u16) << 8 | a as u16);
    }

    fn consume_mbr(&mut self) {
        self.cache.pop_front();
    }

    fn consume_mbr2(&mut self) {
        self.cache.pop_front();
        self.cache.pop_front();
    }
}

impl Default for Ifu {
    fn default() -> Self {
        Self {
            cache: VecDeque::with_capacity(8),
            imar: 0,
        }
    }
}

#[derive(Debug)]
pub struct SysRegs {
    pub lv: SharedReg<u32>,
    pub cpp: SharedReg<u32>,
}

impl SysRegs {
    pub fn new() -> Self {
        SysRegs::default()
    }
}

impl Default for SysRegs {
    fn default() -> Self {
        SysRegs {
            lv: SharedReg::default(),
            cpp: SharedReg::new(RAM_ADDRS as u32 - 1),
        }
    }
}

#[derive(Debug, Default)]
pub struct GenRegs {
    regs: [SharedReg<u32>; 16],
}

impl GenRegs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: usize) -> Option<u32> {
        self.regs.get(id).map(|reg| reg.get())
    }

    pub fn set(&self, id: usize, v: u32) {
        self.regs[id].set(v);
    }
}

#[derive(Debug, Default)]
pub struct Registers {
    pub mem: MemRegs,
    pub sys: SysRegs,
    pub gen: GenRegs,
}

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(regs: &Registers) -> Self {
        let mem = MemRegs {
            mar: regs.mem.mar.clone(),
            mdr: regs.mem.mdr.clone(),
            pc: regs.mem.pc.clone(),
            mbr: regs.mem.mbr.clone(),
            mbr2: regs.mem.mbr2.clone(),
            ifu: Arc::clone(&regs.mem.ifu),
        };
        let sys = SysRegs {
            lv: regs.sys.lv.clone(),
            cpp: regs.sys.cpp.clone(),
        };
        let gen = GenRegs {
            regs: regs.gen.regs.clone(),
        };

        Self { mem, sys, gen }
    }
}
