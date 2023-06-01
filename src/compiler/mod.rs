use log::debug;
use crate::compiler::function::Function;
use crate::compiler::token::Token;
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

        // create a new program
        let mut p = Program::new();

        // Tokenize Code
        let script: Vec<Token> = frontend::parser::script(source).map_err(|e| e.to_string())?;

        // compile functions
        for token in script {

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
                Token::Function(name, args, body) => {

                    // create a new function
                    let func = Function::new(args.clone(), body.clone());

                    // add the function to the global lookup
                    p.globals.insert(name.to_string(), Value::FunctionPointer(p.instructions.len()));

                    // add the function to the program
                    p.instructions.extend(func.instructions);

                    // loop through the anonymous functions
                    for (name, instructions) in func.anon_functions.iter() {
                        p.globals.insert(name.to_string(), Value::FunctionPointer(p.instructions.len()));
                        p.instructions.extend(instructions.clone());
                    }

                },
                _ => {},
            }

        }

        Ok(p)
    }


}