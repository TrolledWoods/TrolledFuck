use std::collections::HashMap;

const MEM_BUF_SIZE_BYTES: usize = 12;
const MEM_FLAGGER: usize = MEM_BUF_SIZE - 1;
const MEM_BUF_SIZE: usize = 1 << MEM_BUF_SIZE_BYTES;
pub struct Memory {
    memory: HashMap<usize, [u8; MEM_BUF_SIZE]>
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: HashMap::new()
        }
    }

    pub fn set(&mut self, loc: isize, value: u8) {
        let loc = (loc & 0x7F_FF_FF_FF) as usize;
        let buf_loc = loc >> MEM_BUF_SIZE_BYTES;
        if let Some(memory) = self.memory.get_mut(&buf_loc) {
            memory[loc & MEM_FLAGGER] = value;
        }else{
            let mut new_buf = [0; MEM_BUF_SIZE];
            new_buf[loc & MEM_FLAGGER] = value;
            self.memory.insert(buf_loc, new_buf);
        }
    }

    pub fn get(&self, loc: isize) -> u8 {
        let loc = (loc & 0x7F_FF_FF_FF) as usize;
        if let Some(memory) = self.memory.get(&(loc >> MEM_BUF_SIZE_BYTES)) {
            memory[loc & MEM_FLAGGER]
        }else{
            0x00
        }
    }

    pub fn modify<F>(&mut self, loc: isize, func: F)
            where F: FnOnce(u8) -> u8 {
        self.set(loc, func(self.get(loc)));
    }
}
