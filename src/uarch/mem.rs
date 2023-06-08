use std::{
    cell::Cell,
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

#[derive(Debug)]
pub struct CltrStoreBuilder {
    firmware: [u64; u8::MAX as usize + 1],
}

impl CltrStoreBuilder {
    /// get the nth word of the memory
    pub fn get(&self, n: u8) -> u64 {
        self.firmware[n as usize]
    }

    /// set the nth word of the memory to `v`
    pub fn set(&mut self, n: u8, v: u64) {
        self.firmware[n as usize] = v
    }

    /// load the microintructions of `v` starting at the nth memory word
    pub fn load<T: IntoIterator<Item = u64>>(&mut self, n: u8, v: T) {
        for (i, mi) in v.into_iter().enumerate() {
            self.firmware[i + n as usize] = mi
        }
    }

    /// Build a `CtrlStore`
    pub fn build(self) -> CtrlStore {
        CtrlStore {
            firmware: Arc::new(self.firmware),
        }
    }
}

#[derive(Debug)]
pub struct CtrlStore {
    firmware: Arc<[u64; u8::MAX as usize + 1]>,
}

impl CtrlStore {
    pub fn new() -> CltrStoreBuilder {
        CltrStoreBuilder {
            firmware: [0; u8::MAX as usize + 1],
        }
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

#[derive(Debug)]
pub struct MemRegs {
    mar: Reg<u32>,
    mdr: Reg<u32>,
    pc: Reg<u32>,
    mbr: Reg<u8>,
    mbr2: Reg<u16>,
}

#[derive(Debug)]
pub struct SysRegs {
    lv: Reg<u32>,
    sp: Reg<u32>,
    cpp: Reg<u32>,
}

#[derive(Debug)]
pub struct GenRegs {
    oa: Reg<u32>,
    ob: Reg<u32>,
    sor: SharedReg<u32>,
}

#[derive(Debug)]
struct Registers {
    mem: MemRegs,
    sys: SysRegs,
    gen: GenRegs,
}
