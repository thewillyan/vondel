#[derive(Debug)]
pub struct Ram {
    data: [u32; u32::MAX as usize + 1],
}

impl Ram {
    // get the nth word of the memory
    pub fn get(&self, n: u32) -> u32 {
        self.data[n as usize]
    }

    // set the nth word of the memory to `v`
    pub fn set(&mut self, n: u16, v: u32) {
        self.data[n as usize] = v
    }
}

#[derive(Debug)]
pub struct CtrlStore {
    firmware: [u64; u8::MAX as usize + 1],
}

impl CtrlStore {
    // get the nth word of the memory
    pub fn get(&self, n: u8) -> u64 {
        self.firmware[n as usize]
    }

    // set the nth word of the memory to `v`
    pub fn set(&mut self, n: u8, v: u64) {
        self.firmware[n as usize] = v
    }
}
