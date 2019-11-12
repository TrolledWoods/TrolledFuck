use crate::Modifiers;
use crate::instructions::{ SHIFT_LEFT, SHIFT_RIGHT, INCREMENT, DECREMENT, READ, PRINT, LOOP_OPEN, LOOP_CLOSE };
use crate::Memory;

pub fn execute_bf(bf: &Vec<u8>, modifiers: &Modifiers) {
    let mut stdin = modifiers.std_in.clone();
    
    let mut memory = Memory::new();
    let mut instr_ptr = 0usize;
    let mut mem_ptr = 0isize;

    while instr_ptr < bf.len() {
        if modifiers.is_debug {
            print!("instr: {}, mem: {} | ", instr_ptr, mem_ptr);
        }
        match bf[instr_ptr] {
            SHIFT_LEFT => {
                if modifiers.is_debug { println!("SHIFT_LEFT"); }
                mem_ptr -= 1;
                instr_ptr += 1;       
            },
            SHIFT_RIGHT => {
                if modifiers.is_debug { println!("SHIFT_RIGHT"); }
                mem_ptr += 1;
                instr_ptr += 1;
            },
            INCREMENT => {
                if modifiers.is_debug { println!("INCREMENT"); }
                memory.modify(mem_ptr, |b| b.wrapping_add(0x01));
                instr_ptr += 1;
            },
            DECREMENT => {
                if modifiers.is_debug { println!("DECREMENT"); }
                memory.modify(mem_ptr, |b| b.wrapping_add(0xff));
                instr_ptr += 1;
            },
            READ => {
                if modifiers.is_debug { println!("READ"); }
                while stdin.len() == 0 {
                    println!("The program requests some more characters to process: ");
                    std::io::stdin().read_line(&mut stdin).expect("Couldn't read line for some reason");
                    stdin = String::from(stdin.trim_end());
                }

                let c = stdin.remove(0);
                if !c.is_ascii() {
                    panic!("Expected ascii character in stdin");
                }

                memory.set(mem_ptr, c as u8);
                instr_ptr += 1;
            },
            PRINT => {
                if modifiers.is_debug { println!("PRINT"); 
                    println!("Output: '{}'", memory.get(mem_ptr) as char);
                }else{
                    print!("{}", memory.get(mem_ptr) as char);
                }
                instr_ptr += 1;
            },
            LOOP_OPEN => {
                if memory.get(mem_ptr) != 0 {
                    if modifiers.is_debug { println!("LOOP_OPEN, entering loop"); }
                    instr_ptr += 5;
                }else{
                    let offset: usize = ((bf[instr_ptr + 1] as u32)).wrapping_add
                                        ((bf[instr_ptr + 2] as u32) << 8).wrapping_add 
                                        ((bf[instr_ptr + 3] as u32) << 16).wrapping_add
                                        ((bf[instr_ptr + 4] as u32) << 24) as usize;

                    if modifiers.is_debug { println!("LOOP_OPEN, exiting loop, offset: {}", offset); }
                    instr_ptr += offset;
                }
            },
            LOOP_CLOSE => {
                if memory.get(mem_ptr) == 0 {
                    if modifiers.is_debug { println!("LOOP_CLOSE, exiting loop"); }
                    instr_ptr += 5;
                }else{
                    let offset = (
                                    ((bf[instr_ptr + 1] as u32)) + 
                                    ((bf[instr_ptr + 2] as u32) << 8) +  
                                    ((bf[instr_ptr + 3] as u32) << 16) +
                                    ((bf[instr_ptr + 4] as u32) << 24)
                                    ) as usize;

                    if modifiers.is_debug { println!("LOOP_CLOSE, continuing loop, offset: {}", offset); }
                    instr_ptr -= offset;
                }
            },
            _ => {
                panic!("Invalid instruction!");
            }
        }
    }
}