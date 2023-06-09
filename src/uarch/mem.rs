use std::{
    cell::Cell,
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Ram {
    data: Arc<Mutex<[u32; u32::MAX as usize + 1]>>,
}

impl Ram {
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
            data: Arc::new(Mutex::new([0; u32::MAX as usize + 1])),
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
    firmware: [u64; 512],
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
            firmware: [0; 512],
            mpc: 0,
        }
    }
}

#[derive(Debug)]
pub struct CtrlStore {
    firmware: Arc<[u64; 512]>,
    pub mpc: SharedReg<u16>,
}

impl CtrlStore {
    /// TERMINATE instruction
    pub const TERM: u64 = 0b11111111111111111111111111111111111111;

    pub fn builder() -> CtrlStoreBuilder {
        CtrlStoreBuilder::default()
    }

    /// Get the next Microinstruction from the CtrlStore, in other words,
    /// fetch the MI at the position stored at MPC.
    ///
    /// ### MI (38 bits):
    ///
    ///```
    /// |   NEXT  |JAM|   ALU  | C BUS  |MEM| A  | B |
    /// |---------|---|--------|--------|---|----|---|
    /// |000000000|000|00000000|00000000|000|0000|000|
    /// ```
    pub fn get_mi(&self) -> u64 {
        self.firmware[self.mpc.get() as usize]
    }

    /// Update the MPC from the opcode of the format:
    ///
    /// `[ ... | NEXT_ADDR | JMPC | JAMN | JAMZ ]`
    ///
    /// where `JMPC`, `JAMN` and `JAMZ` are 1-bit wide and `NEXT_ADDR` is
    /// 9-bit wide. The 4 bits represented by `...` are ignored.
    pub fn update_mpc(&self, mut opcode: u16, z: bool, mbr: u8) {
        // ignored 4 MSBs
        opcode &= 0b0000111111111111;

        let jamz = (opcode & 1) == 1;
        opcode >>= 1;
        let jamn = (opcode & 1) == 1;
        opcode >>= 1;
        let jmpc = (opcode & 1) == 1;
        opcode >>= 1;

        let mut next_addr = opcode;

        if jamn && !z {
            next_addr |= 0b0000000100000000;
        }

        if jamz && z {
            next_addr |= 0b0000000100000000;
        }

        if jmpc {
            next_addr |= mbr as u16;
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
    mar: Reg<u32>,
    mdr: Reg<u32>,
    pc: Reg<u32>,
    mbr: Reg<u8>,
    mbr2: Reg<u16>,
    ifu: Ifu,
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
        self.ifu.consume_mbr();
        self.pc.set(self.pc.get() + 1);
        self.mbr.get()
    }

    pub fn mbr2(&mut self) -> u16 {
        self.ifu.consume_mbr2();
        self.pc.set(self.pc.get() + 2);
        self.mbr2.get()
    }

    pub fn read(&mut self, mem: Ram) {
        self.mdr.set(mem.get(self.mar.get()));
    }

    pub fn write(&mut self, mut mem: Ram) {
        mem.set(self.mar.get(), self.mdr.get())
    }

    pub fn fetch(&mut self, mem: Ram) {
        self.ifu.load(self.mbr.clone(), self.mbr2.clone(), mem);
    }

    pub fn update_pc(&mut self, v: u32, mem: Ram) {
        self.pc.set(v);
        self.ifu.imar = v;
        self.ifu.load(self.mbr.clone(), self.mbr2.clone(), mem);
    }

    pub fn update_mar(&mut self, v: u32) {
        self.mar.set(v)
    }
}

/// A Instruction Fetch Unit with 8 bytes of cache
#[derive(Debug)]
struct Ifu {
    cache: VecDeque<u8>,
    imar: u32,
}

impl Ifu {
    /// fetches a word (4 bytes) from the memory if necessary
    fn try_fetch(&mut self, mem: Ram) {
        if self.cache.len() <= 4 {
            let word = mem.get(self.imar);
            self.imar += 1;
            for &b in word.to_le_bytes().iter().rev() {
                self.cache.push_back(b);
            }
        }
    }

    fn load(&mut self, mbr: Reg<u8>, mbr2: Reg<u16>, mem: Ram) {
        self.try_fetch(mem);
        let a = *self
            .cache
            .front()
            .expect("Should not panic: the memory was fetched previously");
        let b = *self
            .cache
            .get(1)
            .expect("Should not panic: the memory was fetched previously");
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

#[derive(Debug, Default)]
pub struct SysRegs {
    pub lv: Reg<u32>,
    pub sp: Reg<u32>,
    pub cpp: Reg<u32>,
}

impl SysRegs {
    pub fn new() -> Self {
        SysRegs::default()
    }
}

#[derive(Debug, Default)]
pub struct GenRegs {
    pub oa: Reg<u32>,
    pub ob: Reg<u32>,
    pub sor: SharedReg<u32>,
}

impl GenRegs {
    pub fn new() -> Self {
        Self::default()
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
}
