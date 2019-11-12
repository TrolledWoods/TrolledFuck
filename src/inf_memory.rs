const HALF_MEM_BUF_SIZE: usize = 1024;
const MEM_BUF_SIZE: usize = HALF_MEM_BUF_SIZE * 2;
pub struct Memory {
    memory: [u8; MEM_BUF_SIZE]
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0; MEM_BUF_SIZE]
        }
    }

    pub fn set(&mut self, loc: isize, value: u8) {
        self.memory[(loc + HALF_MEM_BUF_SIZE as isize) as usize] = value;
    }

    pub fn get(&self, loc: isize) -> u8 {
        self.memory[(loc + HALF_MEM_BUF_SIZE as isize) as usize]
    }

    pub fn modify<F>(&mut self, loc: isize, func: F)
            where F: FnOnce(u8) -> u8 {
        self.memory[(loc + HALF_MEM_BUF_SIZE as isize) as usize] = func(self.memory[(loc + HALF_MEM_BUF_SIZE as isize) as usize]);
    }
}
