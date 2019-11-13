extern crate ansi_term;
use std::env;

pub mod run;
use run::execute_bf;

pub mod inf_memory;
pub use inf_memory::Memory;

pub mod parse_bf;
pub use parse_bf::{ Lexer };

pub mod compiler;
pub use compiler::{ Compiler };

pub mod instructions {
    pub const SHIFT_RIGHT: u8 = 0x00;
    pub const SHIFT_LEFT: u8 = 0x01;
    pub const INCREMENT: u8 = 0x02;
    pub const DECREMENT: u8 = 0x03;
    pub const LOOP_OPEN: u8 = 0x04;
    pub const LOOP_CLOSE: u8 = 0x05;
    pub const PRINT: u8 = 0x06;
    pub const READ: u8 = 0x07;
    pub const DEBUG: u8 = 0x08;
}

pub struct Modifiers {
    is_debug: bool,
    std_in: String,
    print_bin: bool,
    save_bin: Option<String>
}

impl Modifiers {
    pub fn new() -> Modifiers {
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
    
    // Read modifiers
    let mut modifiers = Modifiers::new();
    for modifier in &args[1..] {
        // Modifiers start with '*'
        if modifier.get(0..1).unwrap() != "*" {
            panic!("Expected '*' to start modifier '{}'", modifier);
        }

        // Split at '=', because data passed to modifiers are split with =, 
        // ex: *bin=example.bin
        let mut contents = modifier.get(1..).unwrap().split("=");
        let name = contents.next();

        // Do things individually depending on which modifier it is
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

            // Too much data passed to the modifier, not all was used!
            if let Some(_) = contents.next() {
                panic!("Unexpected number of parameters to modifier, '{}'", modifier);
            }
        }else {
            // There was no name in the modifier
            panic!("Expected modifier name!, '{}'", modifier);
        }
    }

    (path, modifiers)
}

fn main() {
    if ansi_term::enable_ansi_support().is_err() {
        println!("Couldn't enable console color, so you'll be stuck with monocrome..");
    }
    
    // Command line arguments
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    let (path, modifiers) = read_command_line_args(&args);

    // Parse/read the data, different depending on if the file is a
    //      binary or not.
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
            let compiler = Compiler::new();

            let data: Vec<char> = std::fs::read_to_string(path)
                                    .expect("Invalid file")
                                    .chars().collect();
        
            let mut lexer = Lexer::new(data);
            lexer.tokenize(&vec![String::from("src")], &compiler, false).expect("Invalid stuff happened :(");

            if let Ok(std_file) = std::fs::read_to_string("std.bf") {
                let std_data: Vec<char> = std_file.chars().collect();
                let mut std_lexer = Lexer::new(std_data);
                std_lexer.tokenize(&vec![String::from("std")], &compiler, false).expect("Invalid std stuff happened :(");
            }else{
                println!("WARNING: Standard library could not be loaded");
            }

            compiler.finish_compilation().expect("Invalid compilation");
            assert!(compiler.is_done(), "All dependencies couldn't be resolved");

            compiler.get_compiled_value("src").expect("Didn't compile! :(")
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
        use instructions::*;
        match bf[index] {
            DEBUG => {},
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
