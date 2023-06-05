use std::collections::HashMap;
use crate::vm::instructions::Instruction;
use crate::vm::value::Value;

#[derive(Clone, Debug)]
pub struct Program {
    pub(crate) instructions: Vec<Instruction>,
    pub globals: HashMap<String, Value>,
}

impl Program {

    pub fn new() -> Program {
        Program {
            instructions: Vec::new(),
            globals: HashMap::new(),
        }
    }

}