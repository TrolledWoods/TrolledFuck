use std::collections::{ HashMap, HashSet };
use std::sync::{ Mutex };
use crate::parse_bf::{ Token, TokenType };

pub struct Compiler {
    pub id_map: Mutex<HashMap<String, u16>>,
    pub is_finished: Mutex<bool>,
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
            is_finished: Mutex::new(false),
            n_values: Mutex::new(0),
            ready_to_compile: Mutex::new(HashSet::new()),
            compiled: Mutex::new(HashMap::new()),
            not_compiled: Mutex::new(HashMap::new()),
            dependencies: Mutex::new(HashMap::new())
        }
    }

    pub fn deploy_compilation_thread(&self) {
        // std::thread::spawn(
        //     || {
        //         while *self.is_finished.lock().unwrap() {
        //             if let Ok(result) = self.try_compile_one() {
        //                 if !result {
        //                     std::thread::sleep_ms(10);
        //                 }
        //             }
        //         }
        //     }
        // );
    }

    pub fn get_compiled_value(&self, name: &str) -> Option<Vec<u8>> {
        let id = *(self.id_map.lock().unwrap().get(&String::from(name))?);
        Some(self.compiled.lock().unwrap().get(&id)?.clone())
    }

    pub fn is_done(&self) -> bool {
        self.not_compiled.lock().unwrap().len() == 0
    }

    fn try_compile_one(&self) -> Result<bool, String> {
        // Get a value to work with
        let mut id = None;
        {
            // Pick a ready element
            let mut ready = self.ready_to_compile.lock().unwrap();
            if ready.len() > 0 {
                id = Some(ready.iter().next().unwrap().clone());
                ready.remove(&id.unwrap());
            }
        }

        if let Some(id) = id {
            self.compile(id)?;

            Ok(true)
        }else{
            Ok(false)
        }
    }

    pub fn finish_compilation(&self) -> Result<(), String> {
        while self.try_compile_one()? {}
        *self.is_finished.lock().unwrap() = true;

        Ok(())
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

    pub fn add_dependencies(&self, source: u16, dependencies: &HashSet<String>) -> HashSet<u16> {
        let mut unresolved = HashSet::new();
        let mut depend = self.dependencies.lock().unwrap();
        for dependency in dependencies {
            let id = self.get_identifier_or_create(dependency);

            if let Some(vector) = depend.get_mut(&id) {
                // The dependency vector exists in the dependency HashMap, so push onto that
                unresolved.insert(id);
                vector.push(source);
                continue;
            }

            if self.compiled.lock().unwrap().contains_key(&id) {
                // It's actually resolved already! :D
                continue;
            }

            // It didn't exist in the dependencies, so insert a vector of dependencies onto that
            unresolved.insert(id);
            depend.insert(id, vec![source]);
        }

        unresolved
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

                // If we resolved all their dependencies, hooray!! It can now compile properly
                if dependencies.len() == 0 {
                    self.ready_to_compile.lock().unwrap().insert(dependant);
                }
            }
        }

        Ok(())
    }
    
    pub fn add_compilation_unit(&self, name: String, data: Vec<Token>, dependencies: HashSet<String>) {
        let id = self.get_identifier_or_create(&name);
        
        let unresolved_dependencies = self.add_dependencies(id, &dependencies);
        if unresolved_dependencies.len() > 0 {
            self.not_compiled.lock().unwrap().insert(id, (unresolved_dependencies, data));
        }else{
            self.not_compiled.lock().unwrap().insert(id, (unresolved_dependencies, data));
            self.ready_to_compile.lock().unwrap().insert(id);
        }
    }
}

pub fn create_loop(contained_commands: Vec<u8>) -> Vec<u8> {
    use crate::instructions::*;

    let mut contained_commands = contained_commands;
    let offset = contained_commands.len();

    contained_commands.reserve(10);
    contained_commands.push(LOOP_CLOSE);
    contained_commands.push(( offset        & 0xff) as u8);
    contained_commands.push(((offset >> 8 ) & 0xff) as u8);
    contained_commands.push(((offset >> 16) & 0xff) as u8);
    contained_commands.push(((offset >> 24) & 0xff) as u8);


    let offset = offset + 10;
    contained_commands.insert(0, LOOP_OPEN);
    contained_commands.insert(1, (offset & 0xff) as u8);
    contained_commands.insert(2, ((offset >> 8) & 0xff) as u8);
    contained_commands.insert(3, ((offset >> 16) & 0xff) as u8);
    contained_commands.insert(4, ((offset >> 24) & 0xff) as u8);
    

    contained_commands
}

fn set_to_zero(commands: &mut Vec<u8>) {
    commands.append(&mut create_loop(vec![crate::instructions::DECREMENT]));
}

fn compile_str(string: &str) -> Result<Vec<u8>, String> {
    use crate::instructions::*;
    let mut commands = Vec::new();

    for (i, c) in string.chars().enumerate() {
        if !c.is_ascii() {
            return Err(String::from("Non ascii character :("));
        }

        let size = c as u8;

        if i < string.len() - 1 {
            let sqrt = (size as f32).sqrt().floor() as u8;
            commands.push(SHIFT_RIGHT);
            set_to_zero(&mut commands);
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
            commands.append(&mut create_loop(loop_commands));

            commands.push(SHIFT_LEFT);

            let fault = size - sqrt * sqrt;
            for _ in 0..fault {
                commands.push(INCREMENT);
            }
        }else{
            set_to_zero(&mut commands);

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

    Ok(commands)
}

pub fn compile_node(macros: &Compiler, token: &Token, src_path: &String) -> Result<Vec<u8>, String> {
    use crate::instructions::*;
    use TokenType::*;
    match &token.data {
        Debug => Ok(vec![DEBUG]),
        Str(string) => Ok(compile_str(&string[..])?),
        Macro(name) => {
            Ok(macros.get_compiled_value(&name[..]).expect("Dependency wasn't compiled"))
        },
        Loop(sub_tokens) => {
            let mut contents = Vec::new();
            for sub_token in sub_tokens.iter() {
                contents.append(&mut compile_node(macros, sub_token, src_path)?);
            }

            Ok(create_loop(contents))
        },
        ShiftRight(amount) => Ok(vec![SHIFT_RIGHT; *amount as usize]),
        ShiftLeft(amount) => Ok(vec![SHIFT_LEFT; *amount as usize]),
        Increment(amount) => Ok(vec![INCREMENT; *amount as usize]),
        Decrement(amount) => Ok(vec![DECREMENT; *amount as usize]),
        Print => Ok(vec![PRINT]),
        Read => Ok(vec![READ])
    }
}