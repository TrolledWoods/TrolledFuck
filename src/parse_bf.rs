use crate::instructions::*;
use std::collections::HashMap;

pub struct Lexer {
    text: Vec<char>,
    loc: usize
}

impl Lexer {
    pub fn new(text: Vec<char>) -> Lexer {
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
            if c.is_alphabetic() || (identifier.len() >= 1 && c.is_numeric()) || *c == ';' || *c == '_' {
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

    fn create_loop(&mut self, contained_commands: Vec<u8>) -> Vec<u8> {
        let mut contained_commands = contained_commands;
        let offset = contained_commands.len() + 5;

        contained_commands.insert(0, LOOP_OPEN);
        contained_commands.insert(1, (offset & 0xff) as u8);
        contained_commands.insert(2, ((offset >> 8) & 0xff) as u8);
        contained_commands.insert(3, ((offset >> 16) & 0xff) as u8);
        contained_commands.insert(4, ((offset >> 24) & 0xff) as u8);
        
        let offset = offset - 5;
        contained_commands.push(LOOP_CLOSE);
        contained_commands.push(( offset        & 0xff) as u8);
        contained_commands.push(((offset >> 8 ) & 0xff) as u8);
        contained_commands.push(((offset >> 16) & 0xff) as u8);
        contained_commands.push(((offset >> 24) & 0xff) as u8);

        contained_commands
    }

    fn set_to_zero(&mut self, commands: &mut Vec<u8>) {
        commands.append(&mut self.create_loop(vec![DECREMENT]));
    }

    fn parse_value(&mut self, commands: &mut Vec<u8>, macros: &HashMap<String, Vec<u8>>) -> Result<(), String> {
        if let Some(c) = self.text.get(self.loc) {
            self.loc += 1;
            match *c {
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
                '"' => {
                    let mut contents = String::new();
                    self.set_to_zero(commands);
                    while let Some(c) = self.text.get(self.loc) {
                        self.loc += 1;
                        if *c == '"' {
                            for (i, c) in contents.chars().enumerate() {
                                if !c.is_ascii() {
                                    return Err(String::from("Non ascii character :("));
                                }
        
                                let size = c as u8;

                                if i < contents.len() - 1 {
                                    let sqrt = (size as f32).sqrt().floor() as u8;
                                    commands.push(SHIFT_RIGHT);
                                    self.set_to_zero(commands);
                                    for _ in 0..sqrt {
                                        commands.push(INCREMENT);
                                    }

                                    let mut loop_commands = Vec::new();
                                    loop_commands.push(DECREMENT);
                                    loop_commands.push(SHIFT_LEFT);
                                    for _ in 0..sqrt {
                                        loop_commands.push(INCREMENT);
                                    }
                                    loop_commands.push(SHIFT_RIGHT);
                                    commands.append(&mut self.create_loop(loop_commands));

                                    commands.push(SHIFT_LEFT);

                                    let fault = size - sqrt * sqrt;
                                    for _ in 0..fault {
                                        commands.push(INCREMENT);
                                    }
                                }else{
                                    self.set_to_zero(commands);
    
                                    if size >= 0x88 {
                                        // Invert the size
                                        let size = 0xff ^ size;
            
                                        for _ in 0..size {
                                            commands.push(DECREMENT);
                                        }
                                    }else {
                                        for _ in 0..size {
                                            commands.push(INCREMENT);
                                        }
                                    }
                                }

                                commands.push(SHIFT_RIGHT);
                            }

                            return Ok(());
                        }else if *c == '\\' {
                            if let Some(next_c) = self.text.get(self.loc) {
                                self.loc += 1;

                                match *next_c {
                                    'n' => contents.push('\n'),
                                    't' => contents.push('\t'),
                                    _ => return Err(String::from("Invalid character after '\\'"))
                                }
                            }else {
                                return Err(String::from("File ended before '\\' could be resolved"));
                            }
                        }else{
                            contents.push(*c);
                        }
                    }

                    return Err(String::from("Expected '\"' to end string"));
                },
                '[' => {
                    self.read_loop(commands, macros)?;
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

        Ok(())
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
            if *c == ']' {
                self.loc += 1;

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
            }else{
                self.parse_value(commands, macros)?;
            }
        }

        Err(String::from("Expected ']' to end loop"))
    }

    pub fn tokenize(&mut self, terminatable: bool) -> Result<(Vec<u8>, HashMap<String, Vec<u8>>), String> {
        let mut commands = Vec::new();
        let mut macros = HashMap::new();

        while let Some(c) = self.text.get(self.loc) {
            if *c == ':' {
                self.loc += 1;
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
            }else if *c == '}' {
                self.loc += 1;

                if terminatable {
                    break;
                }else{
                    return Err(String::from("Unexpected '}'"));
                }
            }else{
                self.parse_value(&mut commands, &macros)?;
            }
        }

        Ok((commands, macros))
    }
}