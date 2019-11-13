use std::collections::HashSet;
use crate::Compiler;

#[derive(Debug)]
pub enum TokenType {
    Str(String),
    Macro(String),
    Loop(Vec<Token>),
    Increment(u8),
    Decrement(u8),
    ShiftRight(u8),
    ShiftLeft(u8),
    Print,
    Read,
    Debug
}

#[derive(Debug)]
pub struct Token {
    pub src_loc: Loc,
    pub data: TokenType
}

impl Token {
    pub fn new_debug(loc: Loc) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Debug
        }
    } 

    pub fn new_str(loc: Loc, data: String) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Str(data)
        }
    }

    pub fn new_macro(loc: Loc, identifier: String) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Macro(identifier)
        }
    }

    pub fn new_loop(loc: Loc, sub_tokens: Vec<Token>) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Loop(sub_tokens)
        }
    }

    pub fn new_increment(loc: Loc, n_times: u8) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Increment(n_times)
        }
    }
    
    pub fn new_decrement(loc: Loc, n_times: u8) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Decrement(n_times)
        }
    }

    pub fn new_shift_right(loc: Loc, n_times: u8) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::ShiftRight(n_times)
        }
    }

    pub fn new_shift_left(loc: Loc, n_times: u8) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::ShiftLeft(n_times)
        }
    }

    pub fn new_print(loc: Loc) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Print
        }
    }

    pub fn new_read(loc: Loc) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Read
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Loc {
    line: usize, 
    _char: usize,
    index: usize
}

impl Loc {
    pub fn zero() -> Loc {
        Loc {
            line: 0,
            _char: 0,
            index: 0
        }
    }

    pub fn add_n_chars(&mut self, amount: usize) {
        self._char += amount;
        self.index += amount;
    }

    pub fn add_n_lines(&mut self, amount: usize) {
        self.line += amount;
        self.index += amount;
    }

    pub fn move_with(&mut self, c: char) {
        self.index += 1;
        
        if c == '\n' {
            self.line += 1;
            self._char = 0;
        }else {
            self._char += 1;
        }
    }
}

impl std::fmt::Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.line, self._char)?;
        Ok(())
    }
}

pub struct Lexer {
    text: Vec<char>,
    loc: Loc
}

impl Lexer {
    pub fn new(text: Vec<char>) -> Lexer {
        Lexer {
            text: text,
            loc: Loc::zero()
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.text.get(self.loc.index) {
            if c.is_whitespace() {
                self.loc.move_with(*c);
            }else{
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> Option<String> {
        let mut identifier = String::new();
        
        while let Some(c) = self.text.get(self.loc.index) {
            if c.is_alphabetic() || (identifier.len() >= 1 && c.is_numeric()) || *c == '/' || *c == '_' || *c == '.' {
                self.loc.add_n_chars(1);
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

    fn parse_value(&mut self, compiler: &Compiler, path: &Vec<String>, commands: &mut Vec<Token>, dependencies: &mut HashSet<String>) -> Result<(), String> {
        if let Some(c) = self.text.get(self.loc.index) {
            self.loc.move_with(*c);
            match *c {
                '!' => {
                    commands.push(Token::new_debug(self.loc));
                },
                '#' => {
                    let mut identifier = self.read_identifier().ok_or_else(|| String::from("Expected identifier"))?;
                    
                    if identifier == "use" {
                        // This just defines a macro that is set to another macro, 
                        // i.e "#use /long/path/name" <=> ":name { #/long/path/name }"
                        self.skip_whitespace();

                        // Get the path of the macro to import into current scope
                        let start = self.loc;
                        let mut identifier = self.read_identifier().ok_or_else(|| String::from("Expected identifier"))?;
                        pathify_identifier(path, &mut identifier)?;

                        // Create some strings that are going to be passed into datastructures later
                        let identifier_dep = String::from(&identifier);
                        let identifier_token = String::from(&identifier);

                        // Figure out the path that the import is going to be set to
                        let mut name = String::from(identifier.split('/').rev().next().unwrap());
                        name.insert(0, '/');
                        name.insert_str(0, &path.join("/")[..]);
                        
                        // Add the macro to the compilers list of things to compile
                        let mut dep = HashSet::with_capacity(1);
                        dep.insert(identifier_dep);
                        compiler.add_compilation_unit(
                                String::from(name), 
                                vec![Token::new_macro(start, identifier_token)], 
                                dep
                            );
                    }else{
                        pathify_identifier(path, &mut identifier)?;

                        dependencies.insert(String::from(&identifier));
                        commands.push(Token::new_macro(self.loc, identifier));
                    }
                },
                '"' => {
                    let mut contents = String::new();
                    let start = self.loc;
                    while let Some(c) = self.text.get(self.loc.index) {
                        self.loc.move_with(*c);
                        if *c == '"' {
                            commands.push(Token::new_str(start, String::from(contents)));
                            break;
                        }else if *c == '\\' {
                            if let Some(next_c) = self.text.get(self.loc.index) {
                                self.loc.move_with(*next_c);

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
                    // return Err(String::from("Expected '\"' to end string"));
                },
                '[' => {
                    self.read_loop(compiler, path, commands, dependencies)?;
                },
                '+' => commands.push(Token::new_increment(self.loc, 1)),
                '-' => commands.push(Token::new_decrement(self.loc, 1)),
                '<' => commands.push(Token::new_shift_left(self.loc, 1)),
                '>' => commands.push(Token::new_shift_right(self.loc, 1)),
                ',' => commands.push(Token::new_read(self.loc)),
                '.' => commands.push(Token::new_print(self.loc)),
                _ => {}
            }
        }

        Ok(())
    }

    // This function also generates the initial command for the loop, so don't worry ;)
    fn read_loop(&mut self, compiler: &Compiler, path: &Vec<String>, commands: &mut Vec<Token>, dependencies: &mut HashSet<String>) -> Result<(), String> {
        let start = self.loc;
        let mut contents = Vec::new();

        while let Some(c) = self.text.get(self.loc.index) {
            if *c == ']' {
                self.loc.move_with(*c);
                commands.push(Token::new_loop(start, contents));
                return Ok(());
            }else{
                self.parse_value(compiler, path, &mut contents, dependencies)?;
            }
        }

        Err(String::from("Expected ']' to end loop"))
    }

    pub fn tokenize(&mut self, name: &Vec<String>, compiler: &Compiler, terminatable: bool) -> Result<(), String> {
        let mut commands = Vec::new();
        let mut dependencies = HashSet::new();

        while let Some(c) = self.text.get(self.loc.index) {
            if *c == ':' {
                self.loc.add_n_chars(1);

                // A macro definition!
                let identifier = self.read_identifier().ok_or_else(|| String::from("Expected identifier"))?;
                if identifier.contains("/") {
                    return Err(String::from("Cannot define a macro with ';' in identifier"));
                }
                self.skip_whitespace();
                let c = self.text.get(self.loc.index)
                    .ok_or_else(|| String::from("Unexpected end of file, expected '{'"))?;
                if *c != '{' {
                    return Err(String::from("Expected '{'"));
                }
                self.loc.move_with(*c);

                let mut sub_name = name.clone();
                sub_name.push(identifier);
                self.tokenize(&sub_name, compiler, true)?;
            }else if *c == '}' {
                self.loc.add_n_chars(1);

                if terminatable {
                    break;
                }else{
                    return Err(String::from("Unexpected '}'"));
                }
            }else{
                self.parse_value(compiler, name, &mut commands, &mut dependencies)?;
            }
        }

        compiler.add_compilation_unit(name.join("/"), commands, dependencies);

        Ok(())
    }
}

fn pathify_identifier(path: &Vec<String>, identifier: &mut String) -> Result<(), String> {
    if identifier.get(0..1).unwrap() == "/" {
        identifier.insert_str(0, &path.join("/")[..]);
    }else if identifier.get(0..1).unwrap() == "." {
        let mut n_dots = 0;
        while identifier.get(0..1).ok_or_else(|| String::from("Expected '/' after dots in identifier"))? == "." {
            identifier.remove(0);
            n_dots += 1;
        }

        if identifier.get(0..1).unwrap() != "/" {
            return Err(String::from("Faulty path: Expected '..../', got '....'"));
        }

        let sub_path = &path[0..(path.len() - n_dots)];
        for (i, elem) in sub_path.iter().enumerate() {
            if i > 0 {
                identifier.insert(0, '/');
            }
            identifier.insert_str(0, elem);
        }
    }

    Ok(())
}