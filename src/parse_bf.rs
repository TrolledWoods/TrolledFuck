use crate::instructions::*;
use std::collections::{ HashMap, HashSet };
use std::sync::{ Mutex };

pub struct Compiler {
    pub id_map: Mutex<HashMap<String, u16>>,
    pub ready_to_compile: Mutex<HashSet<u16>>,
    pub n_values: Mutex<u16>,
    pub compiled: Mutex<HashMap<u16, Vec<u8>>>,
    pub not_compiled: Mutex<HashMap<u16, (HashSet<u16>, Vec<Token>)>>,
    pub dependencies: Mutex<HashMap<u16, Vec<u16>>>
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            id_map: Mutex::new(HashMap::new()),
            n_values: Mutex::new(0),
            ready_to_compile: Mutex::new(HashSet::new()),
            compiled: Mutex::new(HashMap::new()),
            not_compiled: Mutex::new(HashMap::new()),
            dependencies: Mutex::new(HashMap::new())
        }
    }

    pub fn is_done(&self) -> bool {
        self.not_compiled.lock().unwrap().len() == 0
    }

    pub fn get_identifier_or_create(&self, identifier: &String) -> u16 {
        let mut id_map = self.id_map.lock().unwrap();
        if let Some(id) = id_map.get(identifier) {
            *id
        }else{
            let mut n_values = self.n_values.lock().unwrap();
            let new_id = *n_values;
            *n_values += 1;
            id_map.insert(String::from(identifier), new_id);
            new_id
        }
    }

    pub fn add_dependencies(&self, source: u16, dependencies: &Vec<String>) -> u16 {
        let mut n_unresolved = 0;
        let mut depend = self.dependencies.lock().unwrap();
        for dependency in dependencies {
            let id = self.get_identifier_or_create(dependency);

            if let Some(vector) = depend.get_mut(&id) {
                // The dependency vector exists in the dependency HashMap, so push onto that
                n_unresolved += 1;
                vector.push(source);
                continue;
            }

            if self.compiled.lock().unwrap().contains_key(&id) {
                // It's actually resolved already! :D
                continue;
            }

            // It didn't exist in the dependencies, so insert a vector of dependencies onto that
            n_unresolved += 1;
            depend.insert(id, vec![source]);
        }

        n_unresolved
    }

    fn compile(&self, element: u16) -> Result<(), String> {
        let (dependencies, ast) = self.not_compiled.lock().unwrap()
                        .remove(&element)
                        .expect("compile: element was an invalid id");
        assert_eq!(dependencies.len(), 0, "Tried compiling element without resolving dependencies first");

        let mut commands = Vec::new();
        for token in ast {
            commands.append(&mut compile_node(self, &token, &String::new())?);
        }

        self.compiled.lock().unwrap().insert(element, commands);
        if let Some(dependants) = self.dependencies.lock().unwrap().remove(&element) {
            for dependant in dependants {
                let mut lock = self.not_compiled.lock().unwrap();
                let (dependencies, _) = lock.get_mut(&dependant)
                                            .expect("compile: Dependant compiled before it's dependency? Makes no sense!");
                dependencies.remove(&element);
            }
        }

        Ok(())
    }
    
    pub fn add_compilation_unit(&self, name: String, data: Vec<Token>, dependencies: Vec<String>) {
        let id = self.get_identifier_or_create(&name);
        
        let unresolved_dependencies = self.add_dependencies(id, &dependencies);
        if unresolved_dependencies > 0 {
            unimplemented!();
        }else{
            self.ready_to_compile.lock().unwrap().insert(id);
        }
    }
}

pub enum TokenType {
    Str(String),
    Macro(String),
    Loop(Vec<Token>),
    Increment(u8),
    Decrement(u8),
    ShiftRight(u8),
    ShiftLeft(u8),
    Print,
    Read
}

pub struct Token {
    pub src_loc: usize,
    pub data: TokenType
}

pub enum NamespaceMember {
    Uncompiled(bool, Vec<Token>),
    Compiled(Vec<u8>)
}

pub fn create_loop(contained_commands: Vec<u8>) -> Vec<u8> {
    let mut contained_commands = contained_commands;
    let offset = contained_commands.len() + 5;

    contained_commands.reserve(10);
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

pub fn compile_node(macros: &Compiler, token: &Token, src_path: &String) -> Result<Vec<u8>, String> {
    use TokenType::*;
    match &token.data {
        Str(string) => unimplemented!(),
        Macro(name) => unimplemented!(),
        Loop(sub_tokens) => {
            let mut contents = Vec::new();
            for sub_token in sub_tokens.iter() {
                contents.append(&mut compile_node(macros, sub_token, src_path)?);
            }

            Ok(create_loop(contents))
        },
        ShiftRight(amount) => Ok(vec![SHIFT_RIGHT]),
        ShiftLeft(amount) => Ok(vec![SHIFT_LEFT]),
        Increment(amount) => Ok(vec![INCREMENT]),
        Decrement(amount) => Ok(vec![DECREMENT]),
        Print => Ok(vec![PRINT]),
        Read => Ok(vec![READ])
    }
}

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