use std::collections::HashMap;
use crate::Compiler;
use crate::Error;

#[derive(Debug, Clone)]
pub enum TokenType {
    Str(String, bool),
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

#[derive(Debug, Clone)]
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

    pub fn new_str(loc: Loc, data: String, is_safe: bool) -> Token {
        Token {
            src_loc: loc,
            data: TokenType::Str(data, is_safe)
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
        write!(f, "({}, {})", self.line + 1, self._char)?;
        Ok(())
    }
}

struct LexerContext {
    errors: Vec<Error>,
    dependencies: HashMap<String, Vec<Loc>>,
    commands: Vec<Token>,
    path: Vec<String>
}

impl LexerContext {
    fn new(path: Vec<String>) -> LexerContext {
        LexerContext {
            errors: Vec::new(),
            dependencies: HashMap::new(),
            commands: Vec::new(),
            path: path
        }
    }

    fn add_dependency(&mut self, string: &str, location: Loc) {
        if let Some(list) = self.dependencies.get_mut(string) {
            list.push(location);
        }else{
            self.dependencies.insert(String::from(string), vec![location]);
        }
    }

    fn add_error(&mut self, loc: Loc, message: String) {
        self.errors.push(Error::new(loc, message));
    }
}

pub struct Lexer {
    text: Vec<char>,
    loc: Loc,
    n_invalid_macro_names: usize
}

impl Lexer {
    pub fn new(text: Vec<char>) -> Lexer {
        Lexer {
            text: text,
            loc: Loc::zero(),
            n_invalid_macro_names: 0
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

    fn parse_char(&mut self, context: &mut LexerContext) -> Option<char> {
        if let Some(c) = self.text.get(self.loc.index) {
            self.loc.move_with(*c);
            if *c == '\\' {
                if let Some(next_c) = self.text.get(self.loc.index) {
                    let start = self.loc;
                    self.loc.move_with(*next_c);

                    match *next_c {
                        'n' => Some('\n'),
                        't' => Some('\t'),
                        _ => {
                            context.add_error(
                                start, String::from("Invalid character after '\\'")
                            );
                            None
                        }
                    }
                }else {
                    context.add_error(
                        self.loc, String::from("File ended before '\\' could be resolved")
                    );
                    None
                }
            }else{
                Some(*c)
            }
        }else{
            None
        }
    }

    fn parse_str(&mut self, context: &mut LexerContext, is_safe: bool) {
        let mut contents = String::new();
        let start = self.loc;
        while let Some(c) = self.parse_char(context) {
            if c == '"' {
                context.commands.push(Token::new_str(start, String::from(contents), is_safe));
                return;
            }else{
                contents.push(c);
            }
        }
        
        context.add_error(self.loc, String::from("Expected '\"' to end string"));
    }

    fn try_parse_number(&mut self, context: &mut LexerContext) -> Option<u8> {
        if let Some(c) = self.text.get(self.loc.index) {
            if *c == '\'' {
                self.loc.add_n_chars(1);
                let c = self.parse_char(context);

                match c {
                    Some(c) => {
                        if c.is_ascii() { 
                            return Some(c as u8);
                        }else{
                            context.add_error(self.loc, String::from("Expected ASCII character"));
                            return None;
                        }
                    },
                    None => {
                        context.add_error(self.loc, String::from("Expected character"));
                        return None;
                    }
                }
            }
        }

        let mut n_digits = 0;
        let mut number = 0u8;

        while let Some(c) = self.text.get(self.loc.index) {
            if let Some(digit) = c.to_digit(16) {
                self.loc.add_n_chars(1);
                n_digits += 1;
                if n_digits <= 2 {
                    number *= 16;
                    number += digit as u8;
                }else{
                    context.add_error(self.loc, String::from("Too big number, expected hexadecimal number with max 2 digits"));
                    return None;
                }
            }else{
                break;
            }
        }

        if n_digits > 0 {
            Some(number)
        }else{
            None
        }
    }

    fn parse_value(
            &mut self, 
            compiler: &Compiler, 
            context: &mut LexerContext) {
        if let Some(c) = self.text.get(self.loc.index) {
            let start = self.loc;
            self.loc.move_with(*c);
            match *c {
                character if character.is_whitespace() => {},
                '🧙' => {
                    println!("{}: Ayoyoyo Wololo!", self.loc);
                },
                ';' => {
                    while let Some(c) = self.text.get(self.loc.index) {
                        self.loc.move_with(*c);
                        if *c == '\n' {
                            return;
                        }
                    }
                },
                '!' => {
                    context.commands.push(Token::new_debug(self.loc));
                },
                '#' => {
                    let identifier_start = self.loc;
                    let mut identifier = match self.read_identifier() {
                        Some(value) => value,
                        None => {
                            context.add_error(start, String::from("Expected identifier"));
                            return;
                        }
                    };
                    
                    if identifier == "use" {
                        // This just defines a macro that is set to another macro, 
                        // i.e "#use /long/path/name" <=> ":name { #/long/path/name }"
                        self.skip_whitespace();

                        // Get the path of the macro to import into current scope
                        let start = self.loc;
                        let mut identifier = match self.read_identifier() {
                            Some(value) => value,
                            None => {
                                context.add_error(start, String::from("Expected identifier"));
                                return;
                            }
                        };

                        if let Err(msg) = pathify_identifier(&context.path, &mut identifier) {
                            context.add_error(start, msg);
                        }

                        // Create some strings that are going to be passed into datastructures later
                        let identifier_dep = String::from(&identifier);
                        let identifier_token = String::from(&identifier);

                        // Figure out the path that the import is going to be set to
                        let mut name = String::from(identifier.split('/').rev().next().unwrap());
                        name.insert(0, '/');
                        name.insert_str(0, &context.path.join("/")[..]);
                        
                        // Add the macro to the compilers list of things to compile
                        let mut dep = HashMap::with_capacity(1);
                        dep.insert(identifier_dep, vec![start]);
                        compiler.add_compilation_unit(
                                String::from(name), 
                                vec![Token::new_macro(start, identifier_token)], 
                                dep
                            );
                    }else{
                        if let Err(msg) = pathify_identifier(&context.path, &mut identifier) {
                            context.add_error(start, msg);
                        }

                        context.add_dependency(&identifier[..], identifier_start);
                        context.commands.push(Token::new_macro(identifier_start, identifier));
                    }
                },
                '(' => {
                    let start = self.loc;
                    let contents_start = context.commands.len();
                    
                    while let Some(c) = self.text.get(self.loc.index) {
                        if *c == ')' {
                            self.loc.move_with(*c);

                            // Get the range of commands in the context that are within the repeat
                            let mut contents = Vec::with_capacity(context.commands.len() - contents_start);
                            while context.commands.len() > contents_start {
                                // .unwrap() is safe since we know the length is larger than 0 
                                // since contents_start has to be >= 0
                                contents.insert(0, context.commands.pop().unwrap());
                            }
                            
                            let count = match self.try_parse_number(context) {
                                Some(value) => value as usize,
                                None => {
                                    context.add_error(self.loc, String::from("Expected number of repitions"));
                                    return;
                                }
                            };

                            for _ in 0..count {
                                for content in contents.iter() {
                                    context.commands.push(content.clone());
                                }
                            }
                            return;
                        }else{
                            self.parse_value(compiler, context);
                        }
                    }

                    context.add_error(start, String::from("Expected ')' to end the repeat block"));
                },
                '0' => {
                    if let Some(c) = self.text.get(self.loc.index) {
                        if *c == '"' {
                            self.loc.move_with(*c);
                            self.parse_str(context, false);
                        }
                    }
                },
                '"' => {
                    self.parse_str(context, true);
                },
                '[' => {
                    let start = self.loc;
                    let contents_start = context.commands.len();
                    
                    while let Some(c) = self.text.get(self.loc.index) {
                        if *c == ']' {
                            self.loc.move_with(*c);

                            // Get the range of commands in the context that are withing the loop
                            let mut contents = Vec::with_capacity(context.commands.len() - contents_start);
                            while context.commands.len() > contents_start {
                                // .unwrap() is safe since we know the length is larger than 0 
                                // since contents_start has to be >= 0
                                contents.insert(0, context.commands.pop().unwrap());
                            }
                            
                            context.commands.push(
                                Token::new_loop(start, contents)
                            );
                            return;
                        }else{
                            self.parse_value(compiler, context);
                        }
                    }

                    context.add_error(start, String::from("Expected ']' to end loop"));
                },
                '+' => {
                    let num = self.try_parse_number(context).unwrap_or(1);
                    context.commands.push(Token::new_increment(self.loc, num));
                },
                '-' => {
                    let num = self.try_parse_number(context).unwrap_or(1);
                    context.commands.push(Token::new_decrement(self.loc, num));
                },
                '<' => {
                    let num = self.try_parse_number(context).unwrap_or(1);
                    context.commands.push(Token::new_shift_left(self.loc, num));
                },
                '>' => {
                    let num = self.try_parse_number(context).unwrap_or(1);
                    context.commands.push(Token::new_shift_right(self.loc, num));
                },
                ',' => context.commands.push(Token::new_read(self.loc)),
                '.' => context.commands.push(Token::new_print(self.loc)),
                _ => {
                    context.add_error(self.loc, String::from(format!("Unexpected token '{}'", *c)))
                }
            }
        }
    }

    pub fn tokenize(&mut self, name: &Vec<String>, compiler: &Compiler, terminatable: bool)
            -> Result<(), Vec<Error>> {
        let mut context = LexerContext::new(name.clone());
        

        while let Some(c) = self.text.get(self.loc.index) {
            let start = self.loc;
            if *c == ':' {
                self.loc.add_n_chars(1);

                // A macro definition!
                let identifier_start = self.loc;
                let identifier = match self.read_identifier() {
                    Some(value) => value,
                    None => {
                        context.add_error(
                            identifier_start, 
                            String::from("Expected an identifier for the macro!"));
                        self.n_invalid_macro_names += 1;
                        "*".repeat(self.n_invalid_macro_names)
                    }
                };

                if identifier.contains("/") {
                    context.add_error(
                        identifier_start, 
                        String::from("Cannot define a macro with '/' in identifier")
                    );
                }
                self.skip_whitespace();
                let opening_bracket_loc = self.loc;
                let c = match self.text.get(self.loc.index) {
                    Some(value) => value,
                    None => {
                        context.add_error(
                            opening_bracket_loc, 
                            String::from("Unexpected end of file, expected macro body definition")
                        );
                        return Err(context.errors);
                    }
                };
                
                if *c != '{' {
                    context.add_error(opening_bracket_loc, String::from("Expected '{'"));
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
                    context.add_error(start, String::from("Unexpected '}'"));
                }
            }else{
                self.parse_value(compiler, &mut context);
            }
        }

        if context.errors.len() > 0 {
            return Err(context.errors);
        }

        compiler.add_compilation_unit(name.join("/"), context.commands, context.dependencies);

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
        for (i, elem) in sub_path.iter().rev().enumerate() {
            if i > 0 {
                identifier.insert(0, '/');
            }
            identifier.insert_str(0, elem);
        }
    }

    Ok(())
}