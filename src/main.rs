use std::env;
use std::collections::HashMap;

struct Modifiers {
    is_debug: bool,
    std_in: String,
    print_bin: bool,
    save_bin: Option<String>
}

impl Modifiers {
    fn new() -> Modifiers {
        Modifiers {
            is_debug: false,
            save_bin: None,
            print_bin: false,
            std_in: String::new()
        }
    }
}

fn read_command_line_args<'a>(args: &'a Vec<String>) -> (&'a str, Modifiers) {
    if args.len() < 1 {
        panic!("Expected at least 1 command line argument; \"File name\"");
    }

    let path = &args[0][..];
    
    let mut modifiers = Modifiers::new();
    for modifier in &args[1..] {
        if modifier.get(0..1).unwrap() != "*" {
            panic!("Expected '*' to start modifier '{}'", modifier);
        }

        let mut contents = modifier.get(1..).unwrap().split("=");
        let name = contents.next();

        if let Some(name) = name {
            match name {
                "debug" => {
                    modifiers.is_debug = true;
                },
                "in" => {
                    let data = contents.next().expect("'in' modifier expected some data, add data after '='");
                    modifiers.std_in.push_str(data);
                },
                "bin" => {
                    let data = contents.next().expect("'bin' modifier expected data");
                    modifiers.save_bin = Some(String::from(data));
                },
                "print_bin" => {
                    modifiers.print_bin = true;
                },
                _ => {
                    panic!("Invalid modifier name, '{}'", name);
                }
            }

            if let Some(_) = contents.next() {
                panic!("Unexpected number of parameters to modifier, '{}'", modifier);
            }
        }else {
            panic!("Expected modifier name!, '{}'", modifier);
        }
    }

    (path, modifiers)
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    let (path, modifiers) = read_command_line_args(&args);

    let data: Vec<u8> = match is_binary(path).unwrap() {
        true => {
            let result = read_bin_from_file(path);
            if let Err(error) = result {
                println!("There was an error, {}", error);
                return;
            }

            result.unwrap()
        },
        false => {
            let data: Vec<char> = std::fs::read_to_string(path)
                                    .expect("Invalid file")
                                    .chars().collect();
        
            let mut lexer = Lexer::new(data);
            match lexer.tokenize(false) {
                Ok((commands, _)) => commands,
                Err(error) => {
                    println!("Error: {}", error);
                    return;
                }
            }
        }
    };

    if let Some(path) = &modifiers.save_bin {
        write_bin_to_file(&path[..], &data)
            .expect("Invalid write bin to file");
    }

    if modifiers.print_bin {
        print_bf_bin(&data);
    }
    execute_bf(&data, &modifiers);
}

fn is_binary(file_name: &str) -> std::io::Result<bool> {
    use std::io::prelude::*;
    use std::fs::File;

    let mut file = File::open(file_name)?;
    let mut data = [0u8; 4];
    if file.read_exact(&mut data).is_err() {
        return Ok(false);
    }

    Ok( data[0] == 0xBF &&
        data[1] == 0xFF &&
        data[2] == 0xBB &&
        data[3] == 0xFF)
}

fn read_bin_from_file(file_name: &str) -> std::io::Result<Vec<u8>> {
    use std::io::prelude::*;
    use std::fs::File;

    let mut file = File::open(file_name)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    file.flush()?;

    let real_data = Vec::from(&data[4..]);
    Ok(real_data)
}

fn write_bin_to_file(file_name: &str, data: &Vec<u8>) -> std::io::Result<()> {
    use std::io::prelude::*;
    use std::fs::File;

    let mut file = File::create(file_name)?;
    file.write(&[0xBF, 0xFF, 0xBB, 0xFF])?;
    file.write(&data[..])?;

    Ok(())
}

fn print_bf_bin(bf: &Vec<u8>) {
    let mut text = String::new();
    let mut index = 0;

    while index < bf.len() {
        match bf[index] {
            SHIFT_LEFT => text.push('<'),
            SHIFT_RIGHT => text.push('>'),
            INCREMENT => text.push('+'),
            DECREMENT => text.push('-'),
            PRINT => text.push('.'),
            READ => text.push(','),
            LOOP_OPEN => {
                text.push('[');
                index += 4;
            },
            LOOP_CLOSE => {
                text.push(']');
                index += 4;
            },
            _ => panic!("print_bf_bin got invalid bf binary")
        }

        index += 1;
    }

    println!("Bin: {}", text);
}

fn execute_bf(bf: &Vec<u8>, modifiers: &Modifiers) {
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

const HALF_MEM_BUF_SIZE: usize = 1024;
const MEM_BUF_SIZE: usize = HALF_MEM_BUF_SIZE * 2;
struct Memory {
    memory: [u8; MEM_BUF_SIZE]
}

impl Memory {
    fn new() -> Memory {
        Memory {
            memory: [0; MEM_BUF_SIZE]
        }
    }

    fn set(&mut self, loc: isize, value: u8) {
        self.memory[(loc + HALF_MEM_BUF_SIZE as isize) as usize] = value;
    }

    fn get(&self, loc: isize) -> u8 {
        self.memory[(loc + HALF_MEM_BUF_SIZE as isize) as usize]
    }

    fn modify<F>(&mut self, loc: isize, func: F)
            where F: FnOnce(u8) -> u8 {
        self.memory[(loc + HALF_MEM_BUF_SIZE as isize) as usize] = func(self.memory[(loc + HALF_MEM_BUF_SIZE as isize) as usize]);
    }
}

const SHIFT_RIGHT: u8 = 0x00;
const SHIFT_LEFT: u8 = 0x01;
const INCREMENT: u8 = 0x02;
const DECREMENT: u8 = 0x03;
const LOOP_OPEN: u8 = 0x04;
const LOOP_CLOSE: u8 = 0x05;
const PRINT: u8 = 0x06;
const READ: u8 = 0x07;

struct Lexer {
    text: Vec<char>,
    loc: usize
}

impl Lexer {
    fn new(text: Vec<char>) -> Lexer {
        Lexer {
            text: text,
            loc: 0
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.text.get(self.loc) {
            if c.is_whitespace() {
                self.loc += 1;
            }else{
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> Option<String> {
        let mut identifier = String::new();
        
        while let Some(c) = self.text.get(self.loc) {
            if c.is_alphabetic() || *c == ';' || *c == '_' {
                self.loc += 1;
                identifier.push(*c);
            }else{
                break;
            }
        }

        if identifier.len() == 0 {
            None
        }else{
            Some(identifier)
        }
    }

    // This function also generates the initial command for the loop, so don't worry ;)
    fn read_loop(&mut self, commands: &mut Vec<u8>, macros: &HashMap<String, Vec<u8>>) -> Result<(), String> {
        let start_loc = commands.len();

        commands.push(LOOP_OPEN);
        commands.push(0); // Temporary values for the offset
        commands.push(0);
        commands.push(0);
        commands.push(0);

        while let Some(c) = self.text.get(self.loc) {
            self.loc += 1;
            match c {
                '#' => {
                    // A macro usage!
                    let identifier = self.read_identifier().ok_or_else(|| String::from("Expected identifier"))?;
                    
                    if let Some(macro_commands) = macros.get(&identifier) {
                        for command in macro_commands.iter() {
                            commands.push(*command);
                        }
                    }else{
                        return Err(String::from("Invalid identifier"));
                    }
                },
                '[' => {
                    self.read_loop(commands, macros)?;
                },
                ']' => {
                    let end_loc = commands.len();
                    let offset = end_loc - start_loc;

                    commands[start_loc + 1] = (offset & 0xff) as u8;
                    commands[start_loc + 2] = ((offset >> 8) & 0xff) as u8;
                    commands[start_loc + 3] = ((offset >> 16) & 0xff) as u8;
                    commands[start_loc + 4] = ((offset >> 24) & 0xff) as u8;

                    
                    let offset = offset - 5;
                    commands.push(LOOP_CLOSE);
                    commands.push(( offset        & 0xff) as u8);
                    commands.push(((offset >> 8 ) & 0xff) as u8);
                    commands.push(((offset >> 16) & 0xff) as u8);
                    commands.push(((offset >> 24) & 0xff) as u8);
                    return Ok(());
                },
                '+' => commands.push(INCREMENT),
                '-' => commands.push(DECREMENT),
                '<' => commands.push(SHIFT_LEFT),
                '>' => commands.push(SHIFT_RIGHT),
                ',' => commands.push(READ),
                '.' => commands.push(PRINT),
                _ => {}
            }
        }

        Err(String::from("Expected ']' to end loop"))
    }

    fn tokenize(&mut self, terminatable: bool) -> Result<(Vec<u8>, HashMap<String, Vec<u8>>), String> {
        let mut commands = Vec::new();
        let mut macros = HashMap::new();

        while let Some(c) = self.text.get(self.loc) {
            self.loc += 1;
            match *c {
                ':' => {
                    // A macro definition!
                    let identifier = self.read_identifier().ok_or_else(|| String::from("Expected identifier"))?;
                    if identifier.contains(";") {
                        return Err(String::from("Cannot define a macro with ';' in identifier"));
                    }
                    self.skip_whitespace();
                    let c = self.text.get(self.loc).ok_or_else(|| String::from("Unexpected end of file, expected '{'"))?;
                    if *c != '{' {
                        return Err(String::from("Expected '{'"));
                    }
                    self.loc += 1;

                    let (sub_commands, sub_macros) = self.tokenize(true)?;

                    for (sub_macro_name, sub_macro_commands) in sub_macros {
                        let mut sub_identifier = identifier.clone();
                        sub_identifier.push(';');
                        sub_identifier.push_str(&sub_macro_name[..]);

                        macros.insert(sub_identifier, sub_macro_commands);
                    }

                    macros.insert(identifier, sub_commands);
                },
                '#' => {
                    // A macro usage!
                    let identifier = self.read_identifier().ok_or_else(|| String::from("Expected identifier"))?;
                    
                    if let Some(macro_commands) = macros.get(&identifier) {
                        for command in macro_commands.iter() {
                            commands.push(*command);
                        }
                    }else{
                        return Err(String::from("Invalid identifier"));
                    }
                },
                '}' => {
                    if terminatable {
                        break;
                    }else{
                        return Err(String::from("Unexpected '}'"));
                    }
                },
                '[' => self.read_loop(&mut commands, &macros)?,
                '+' => commands.push(INCREMENT),
                '-' => commands.push(DECREMENT),
                '<' => commands.push(SHIFT_LEFT),
                '>' => commands.push(SHIFT_RIGHT),
                ',' => commands.push(READ),
                '.' => commands.push(PRINT),
                ']' => return Err(String::from("Unexpected ']'")),
                _ => {}
            }
        }

        Ok((commands, macros))
    }
}