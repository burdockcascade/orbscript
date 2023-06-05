use std::collections::HashMap;
use std::fmt::format;
use log::{debug, info, trace};
use crate::compiler::function::Function;
use crate::compiler::token::Token;
use crate::vm::instructions::Instruction;
use crate::vm::program::Program;
use crate::vm::value::Value;

mod frontend;
mod token;
mod function;

pub struct Compiler {

}

impl Compiler {

    pub fn new() -> Compiler {
        Compiler {

        }
    }

    pub fn compile(&mut self, source: &str) -> Result<Program, String> {

        info!("Compiling program");

        // create a new program
        let mut p = Program::new();

        // Tokenize Code
        let script: Vec<Token> = frontend::parser::script(source).map_err(|e| e.to_string())?;

        trace!("Tokens: {:?}", script);

        // compile globals
        for token in script.clone() {

            match token {
                Token::Constant(name, value) => {
                    match *value {
                        Token::Integer(i) => { p.globals.insert(name.to_string(), Value::Integer(i)); },
                        Token::Float(f) => { p.globals.insert(name.to_string(), Value::Float(f)); },
                        Token::String(s) => { p.globals.insert(name.to_string(), Value::String(s)); },
                        Token::Bool(b) => { p.globals.insert(name.to_string(), Value::Bool(b)); },
                        _ => {}
                    }
                },
                Token::Class(name, body) => {

                    // class template
                    let mut class_def = HashMap::new();

                    // Build template for class
                    for item in body {
                        match item {
                            Token::Function(_, name, args, body) => { class_def.insert(name.to_string(), Value::FunctionRef(format!("{}.{}", name, name))); },
                            Token::Variable(name, value) => { class_def.insert(name.to_string(), Value::Null); }
                            _ => unreachable!("Invalid class body item")
                        }
                    }

                    // add default constructor
                    class_def.insert(name.clone(), Value::Null);

                    // add the class to the global scope
                    p.globals.insert(name.to_string(), Value::Class(class_def));

                },
                Token::Function(parent_class, name, args, body) =>  { p.globals.insert(name.to_string(), Value::FunctionPointer(0)); },
                _ => {},
            }

        }

        // compile functions and update globals
        for token in script.clone() {

            match token {
                Token::Class(class_name, body) => {

                    let mut class_def = HashMap::new();

                    for item in body {
                        match item {

                            // add the function instruction pointer to the class
                            Token::Function(_, name, mut args, body) => {

                                // push 'self' into the arguments
                                args.insert(0, Token::String("self".to_string()));

                                // create a new function
                                let func = Function::new(args.clone(), body.clone(), p.globals.clone());

                                // function name with class
                                let full_class_function_name = format!("{}.{}", class_name, name);

                                // get the position of the function
                                class_def.insert(name.to_string(), Value::FunctionRef(full_class_function_name.clone()));

                                p.globals.insert(full_class_function_name, Value::FunctionPointer(p.instructions.len()));

                                // add the function to the program
                                p.instructions.extend(func.instructions);

                                // loop through the anonymous functions
                                for (name, instructions) in func.anon_functions.iter() {
                                    p.globals.insert(name.to_string(), Value::FunctionPointer(p.instructions.len()));
                                    p.instructions.extend(instructions.clone());
                                }

                            },

                            // add the variable to the class
                            Token::Variable(name, value) => {
                                class_def.insert(name.to_string(), Value::Null);
                            }

                            _ => unreachable!("Invalid class body item")
                        }
                    }

                    p.globals.insert(class_name.to_string(), Value::Class(class_def));

                }
                Token::Function(parent_class, name, args, body) => {

                    // create a new function
                    let func = Function::new(args.clone(), body.clone(), p.globals.clone());

                    // get the position of the function
                    let function_instruction_pointer = Value::FunctionPointer(p.instructions.len());

                    // add the function to the program
                    p.instructions.extend(func.instructions);

                    // loop through the anonymous functions
                    for (name, instructions) in func.anon_functions.iter() {
                        p.globals.insert(name.to_string(), Value::FunctionPointer(p.instructions.len()));
                        p.instructions.extend(instructions.clone());
                    }

                    // if the function is attached to a class
                    if let Some(class_name) = parent_class {

                        // if the class does not exist, create it
                        if !p.globals.contains_key(class_name.as_str()) {
                            p.globals.insert(class_name.to_string(), Value::Class(HashMap::new()));
                        }

                        // get global class and add new entry to it
                        if let Value::Class(class) = p.globals.get(class_name.as_str()).expect("Class should exist") {
                            let mut cc = class.clone();
                            cc.insert(name.to_string(), function_instruction_pointer);
                            p.globals.insert(class_name.to_string(), Value::Class(cc));
                        }


                    } else {

                        // add the function to the global lookup
                        p.globals.insert(name.to_string(), function_instruction_pointer);

                    }

                },
                _ => {},
            }

        }

        return Ok(p);
    }

}