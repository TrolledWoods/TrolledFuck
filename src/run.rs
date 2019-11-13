use crate::Modifiers;
use crate::instructions::{ SHIFT_LEFT, SHIFT_RIGHT, INCREMENT, DECREMENT, READ, PRINT, LOOP_OPEN, LOOP_CLOSE };
use crate::Memory;

fn shift_style()  -> ansi_term::Style { ansi_term::Color::Purple.bold() }
fn modify_style() -> ansi_term::Style { ansi_term::Color::Green .bold() }
fn loop_style()   -> ansi_term::Style { ansi_term::Color::Yellow.bold() }
fn io_style()     -> ansi_term::Style { ansi_term::Color::Cyan  .bold() }

pub fn execute_bf(bf: &Vec<u8>, modifiers: &Modifiers) {
    let mut stdin = modifiers.std_in.clone();
    
    let mut memory = Memory::new();
    let mut instr_ptr = 0usize;
    let mut mem_ptr = 0isize;

    let mut print_buf = String::with_capacity(200);

    while instr_ptr < bf.len() {
        if modifiers.is_debug {
            print!("instr: {:.>4X}, mem: {:.>4X} | ", instr_ptr, mem_ptr);
        }
        match bf[instr_ptr] {
            SHIFT_LEFT => {
                if modifiers.is_debug { println!("{}", shift_style().paint("SHIFT_LEFT")); }
                mem_ptr -= 1;
                instr_ptr += 1;       
            },
            SHIFT_RIGHT => {
                if modifiers.is_debug { println!("{}", shift_style().paint("SHIFT_RIGHT")); }
                mem_ptr += 1;
                instr_ptr += 1;
            },
            INCREMENT => {
                if modifiers.is_debug { println!("{}", modify_style().paint("INCREMENT")); }
                memory.modify(mem_ptr, |b| b.wrapping_add(0x01));
                instr_ptr += 1;
            },
            DECREMENT => {
                if modifiers.is_debug { println!("{}", modify_style().paint("DECREMENT")); }
                memory.modify(mem_ptr, |b| b.wrapping_add(0xff));
                instr_ptr += 1;
            },
            READ => {
                if modifiers.is_debug { println!("{}", io_style().paint("READ")); }
                while stdin.len() == 0 {
                    if print_buf.len() > 0 {
                        println!("{}", &print_buf);
                        print_buf.clear();
                    }

                    println!("{}", 
                        ansi_term::Color::Red
                        .blink()
                        .paint("The program requests some more characters to process: "));
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
                if modifiers.is_debug { 
                    println!("{}: '{}'", modify_style().paint("PRINT"), memory.get(mem_ptr) as char);
                }

                print_buf.push(memory.get(mem_ptr) as char);
                if print_buf.len() >= 100 {
                    print!("{}", print_buf);
                    print_buf.clear();
                }
                instr_ptr += 1;
            },
            LOOP_OPEN => {
                if memory.get(mem_ptr) != 0 {
                    if modifiers.is_debug { 
                        println!("{}, entering loop", loop_style().paint("LOOP_OPEN")); 
                    }
                    instr_ptr += 5;
                }else{
                    let offset: usize = ((bf[instr_ptr + 1] as u32)).wrapping_add
                                        ((bf[instr_ptr + 2] as u32) << 8).wrapping_add 
                                        ((bf[instr_ptr + 3] as u32) << 16).wrapping_add
                                        ((bf[instr_ptr + 4] as u32) << 24) as usize;

                    if modifiers.is_debug { 
                        println!("{}, exiting loop, offset: {}", loop_style().paint("LOOP_OPEN"), offset); }
                    instr_ptr += offset;
                }
            },
            LOOP_CLOSE => {
                if memory.get(mem_ptr) == 0 {
                    if modifiers.is_debug { 
                        println!("{}, exiting loop", loop_style().paint("LOOP_CLOSE")); 
                    }
                    instr_ptr += 5;
                }else{
                    let offset = (
                                    ((bf[instr_ptr + 1] as u32)) + 
                                    ((bf[instr_ptr + 2] as u32) << 8) +  
                                    ((bf[instr_ptr + 3] as u32) << 16) +
                                    ((bf[instr_ptr + 4] as u32) << 24)
                                    ) as usize;

                    if modifiers.is_debug { 
                        println!("{}, continuing loop, offset: {}", loop_style().paint("LOOP_CLOSE"), offset); 
                    }
                    instr_ptr -= offset;
                }
            },
            _ => {
                panic!("Invalid instruction!");
            }
        }
    }
    
    // Print the final printing buffer
    if print_buf.len() > 0 {
        println!("{}", &print_buf);
    }
}