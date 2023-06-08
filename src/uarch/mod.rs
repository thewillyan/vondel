use std::collections::VecDeque;

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

#[derive(Debug)]
struct Ifu {
    cache: VecDeque<u8>,
    imar: u32,
    mem: mem::Ram,
}

impl Ifu {
    // fetches a word (4 bytes) from the memory
    fn fetch(&mut self) {
        let word = self.mem.get(self.imar);
        self.imar += 1;
        for &b in word.to_le_bytes().iter().rev() {
            self.cache.push_back(b);
        }
    }

    fn get_mbr(&mut self) -> u8 {
        if self.cache.len() <= 1 {
            self.fetch();
        }
        self.cache
            .pop_front()
            .expect("Should not panic: memory was fetched previously")
    }

    fn get_mbr2(&mut self) -> u16 {
        if self.cache.len() <= 2 {
            self.fetch();
        }
        let a = self
            .cache
            .pop_front()
            .expect("Should not panic: memory was fetched previously");
        let b = self
            .cache
            .pop_front()
            .expect("Should not panic: memory was fetched previously");
        (b as u16) << 8 | a as u16
    }
}
