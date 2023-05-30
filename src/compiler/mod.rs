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
        debug!("Compiling functions");
        for token in script.iter() {

            match token {
                Token::Function(name, args, body) => {

                    debug!("Compiling function: {}", name);

                    // create a new function
                    let f = Function::new();

                    // compile the function
                    let ins = f.compile(args.clone(), body.clone());

                    // add the function to the global lookup
                    p.globals.insert(name.to_string(), Value::FunctionRef(p.instructions.len()));

                    // add the function to the program
                    p.instructions.extend(ins);

                },
                _ => {},
            }

        }

        Ok(p)
    }


}