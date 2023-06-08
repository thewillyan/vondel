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
    firmware: [u64; u8::MAX as usize + 1],
}

impl CtrlStoreBuilder {
    /// set the nth word of the memory to `v`
    pub fn set(mut self, n: u8, v: u64) -> Self {
        self.firmware[n as usize] = v;
        self
    }

    /// load the microintructions of `v` starting at the nth memory word
    pub fn load<T: IntoIterator<Item = u64>>(mut self, n: u8, v: T) -> Self {
        for (i, mi) in v.into_iter().enumerate() {
            self.firmware[i + n as usize] = mi;
        }
        self
    }

    /// Build a `CtrlStore`
    pub fn build(self) -> CtrlStore {
        CtrlStore {
            firmware: Arc::new(self.firmware),
        }
    }
}

impl Default for CtrlStoreBuilder {
    fn default() -> Self {
        CtrlStoreBuilder {
            firmware: [0; u8::MAX as usize + 1],
        }
    }
}

#[derive(Debug)]
pub struct CtrlStore {
    firmware: Arc<[u64; u8::MAX as usize + 1]>,
}

impl CtrlStore {
    pub fn builder() -> CtrlStoreBuilder {
        CtrlStoreBuilder::default()
    }

    // get the nth word of the memory
    pub fn get(&self, n: u8) -> u64 {
        self.firmware[n as usize]
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

#[derive(Debug, Clone)]
pub struct SharedReg<T> {
    v: Arc<Mutex<T>>,
}

impl<T: Default> Default for SharedReg<T> {
    fn default() -> Self {
        let v = Arc::new(Mutex::new(T::default()));
        Self { v }
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
    pub fn mar(&self) -> u32 {
        self.mar.get()
    }

    pub fn mdr(&self) -> u32 {
        self.mdr.get()
    }

    pub fn pc(&self) -> u32 {
        self.pc.get()
    }

    pub fn mbr(&self) -> u8 {
        self.mbr.get()
    }

    pub fn mbr2(&self) -> u16 {
        self.mbr2.get()
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, mem: Ram) {
        self.mdr.set(mem.get(self.mar.get()));
    }

    pub fn write(&mut self, mut mem: Ram) {
        mem.set(self.mar.get(), self.mdr.get())
    }

    pub fn fetch_mbr(&mut self, mem: Ram) {
        self.mbr.set(self.ifu.get_mbr(mem));
        self.pc.set(self.pc.get() + 1)
    }

    pub fn fetch_mbr2(&mut self, mem: Ram) {
        self.mbr2.set(self.ifu.get_mbr2(mem));
        self.pc.set(self.pc.get() + 2);
    }

    pub fn update_pc(&mut self, v: u32) {
        self.pc.set(v);
        self.ifu.imar = v
    }
}

/// A Instruction Fetch Unit with 8 bytes of cache
#[derive(Debug)]
struct Ifu {
    cache: VecDeque<u8>,
    imar: u32,
}

impl Ifu {
    pub fn new() -> Self {
        Self::default()
    }

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

    fn get_mbr(&mut self, mem: Ram) -> u8 {
        self.try_fetch(mem);
        self.cache
            .pop_front()
            .expect("Should not panic: the memory was fetched previously")
    }

    fn get_mbr2(&mut self, mem: Ram) -> u16 {
        self.try_fetch(mem);
        let a = self
            .cache
            .pop_front()
            .expect("Should not panic: the memory was fetched previously");
        let b = self
            .cache
            .pop_front()
            .expect("Should not panic: the memory was fetched previously");
        (b as u16) << 8 | a as u16
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
    lv: Reg<u32>,
    sp: Reg<u32>,
    cpp: Reg<u32>,
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
