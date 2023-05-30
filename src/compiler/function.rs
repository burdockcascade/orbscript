use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use log::{debug, trace};
use crate::compiler::token::Token;
use crate::vm::instructions::Instruction;
use crate::vm::value::Value;

pub struct Function {
    instructions: Vec<Instruction>,
    variables: HashMap<String, usize>
}

impl Function {
    pub fn new() -> Function {
        Function {
            instructions: vec![],
            variables: Default::default(),
        }
    }

    pub fn compile(mut self, parameters: Vec<Token>, body: Vec<Token>) -> Vec<Instruction> {

        // if there are no statements then return
        if body.is_empty() {
            return vec![Instruction::Return(false)];
        }

        // store the parameters as variables
        self.add_parameters(parameters.clone());

        // compile the statements
        self.compile_statements(body);

        // if tha last instruction is not a return then add one
        if matches!(self.instructions.last(), Some(Instruction::Return(_))) == false {
            self.instructions.push(Instruction::Return(false));
        }

        self.instructions

    }

    //==============================================================================================
    // STATEMENTS

    // compile a list of statements
    fn compile_statements(&mut self, statements: Vec<Token>) {
        for statement in statements {
            self.compile_statement(Box::new(statement));
        }
    }

    // compile a statement
    fn compile_statement(&mut self, statement: Box<Token>)  {
        match *statement {
            Token::Assert(exp) => self.compile_assert(exp),
            Token::Print(exp) => self.compile_print(exp),
            Token::Variable(name, expr) => { self.compile_variable(name, expr); },
            Token::Assign(name, expr) => self.compile_assignment(name, expr),
            Token::Call(name, args) => self.compile_call(name, args),
            Token::Return(expr) => self.compile_return(expr),
            Token::WhileLoop(cond, body) => self.compile_while_loop(cond, body),
            Token::ForI(var, start, step, end, body) => self.compile_for_loop(var, start, step, end, body),
            Token::IfElse(cond, body, else_body) => self.compile_if_else(cond, body, else_body),
            Token::Comment(_) => { },
            _ => unimplemented!("statement not implemented: {:?}", statement)
        }
    }

    fn compile_assert(&mut self, exp: Box<Token>) {
        self.compile_expression(exp);
        self.instructions.push(Instruction::Assert);
    }

    // compile a print statement
    fn compile_print(&mut self, exp: Box<Token>) {
        self.compile_expression(exp);
        self.instructions.push(Instruction::Print);
    }


    //==============================================================================================
    // VARIABLES

    fn add_parameters(&mut self, parameters: Vec<Token>) {
        for param in parameters {
            self.add_variable(param.to_string());
        }
    }

    fn compile_variable(&mut self, name: Box<Token>, expr: Box<Token>) -> usize {

        // Declare variable
        let slot = self.add_variable(name.to_string());

        // compile the value
        self.compile_expression(expr);

        // store the value
        self.instructions.push(Instruction::MoveToLocalVariable(slot));

        slot
    }

    fn compile_tmp_variable(&mut self, expr: Box<Token>) -> usize {

        // create tmp variable name
        let name = format!("tmp{}", self.variables.len());

        // Declare variable
        let slot = self.add_variable(name);

        // compile the value
        self.compile_expression(expr);

        // store the value
        self.instructions.push(Instruction::MoveToLocalVariable(slot));

        slot
    }

    // compile assignment
    fn compile_assignment(&mut self, left: Box<Token>, right: Box<Token>) {

        debug!("compiling assignment {:?} = {:?}", left, right);

        match *left.clone() {

            // store value in variable
            Token::Identifier(name) => {
                trace!("storing value in variable {}", name.to_string());

                // get the variable slot
                let slot = self.get_variable(name.as_str());

                // compile the value
                self.compile_expression(right);

                // store the value
                self.instructions.push(Instruction::MoveToLocalVariable(slot));
            },

            // Token::DotChain(start, mut chain) => {
            //
            //     // remove last item from chain
            //     let last_item = chain.pop().expect("chain to have at least one item");
            //
            //     self.compile_chain(&start, chain.as_slice());
            //     self.compile_expression(right);
            //
            //     match last_item {
            //         Token::Identifier(name) => {
            //             self.instructions.push(Instruction::StackPush(Value::String(name.to_string())));
            //             self.instructions.push(Instruction::SetCollectionItemByKey);
            //         },
            //
            //         // fixme
            //         Token::ArrayIndex(name, index) => {
            //             self.instructions.push(Instruction::StackPush(Value::String(name.to_string())));
            //             self.instructions.push(Instruction::SetCollectionItemByKey);
            //         },
            //         _ => panic!("last item in chain is not a variable or index")
            //     }
            //
            // },

            // store value in array index
            Token::CollectionIndex(name, index) => {
                trace!("storing value in index {:?} of {}", index, name.to_string());

                // load the variable
                let Token::Identifier(name_as_string) = *name else { panic!("name is not an identifier") };
                let slot = self.get_variable(name_as_string.as_str());
                self.instructions.push(Instruction::LoadLocalVariable(slot));

                // compile the value
                self.compile_expression(right);

                // compile the index
                self.compile_expression(index);

                // add value to collection
                self.instructions.push(Instruction::SetCollectionItemByKey);

                // update variable
                self.instructions.push(Instruction::MoveToLocalVariable(slot));
            },

            _ => panic!("name is not an identifier or index")
        }

    }

    //==============================================================================================
    // FUNCTIONS

    // compile a function call
    fn compile_call(&mut self, name: Box<Token>, args: Vec<Token>) {

        let arg_len = args.len();
        let function_name = name.to_string();

        trace!("call to function '{:?}' with {} args", function_name, arg_len);

        self.instructions.push(Instruction::LoadGlobal(function_name));

        // compile the arguments
        for arg in args {
            self.compile_expression(Box::new(arg));
        }

        self.instructions.push(Instruction::Call(arg_len));
    }

    // compile a return statement
    fn compile_return(&mut self, expr: Box<Token>) {
        self.compile_expression(expr);
        self.instructions.push(Instruction::Return(true));
    }

    //==============================================================================================
    // IF

    // compile if statement
    fn compile_if_else(&mut self, expr: Box<Token>, then_body: Vec<Token>, else_body: Option<Vec<Token>>) {
        trace!("compiling ifelse");

        // Compile If Statement
        self.compile_expression(expr);

        // Jump to Else if not True
        let jump_to_else= self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("no where to jump to")));

        // Compile Statements for True
        self.compile_statements(then_body);
        let jump_to_end= self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("can not jump tot end")));

        // Update Else Jump
        let jump_to_pos = self.instructions.len() - jump_to_else;
        self.instructions[jump_to_else] = Instruction::JumpIfFalse(jump_to_pos as i32);

        match else_body {
            None => {}
            Some(els) => {
                let _ = self.compile_statements(els);
            }
        }

        // Update Jump to End
        self.instructions[jump_to_end] = Instruction::JumpForward(self.instructions.len() - jump_to_end);
    }
    
    //==============================================================================================
    // LOOPS

    // compile for loop
    fn compile_for_loop(&mut self, var: Box<Token>, start: Box<Token>, end: Box<Token>, step: Box<Token>, block: Vec<Token>) {

        trace!("for loop starting at {:?} and ending at {:?} with step {:?}", start, end, step);

        // set variable to initial value
        let var_slot = self.compile_variable(var, start);
        let end_slot = self.compile_tmp_variable(end.clone());
        let step_slot = self.compile_tmp_variable(step.clone());

        // Mark instruction pointer
        let start_of_loop = self.instructions.len();

        // Check if var is less than end
        self.instructions.push(Instruction::LoadLocalVariable(var_slot));
        self.instructions.push(Instruction::LoadLocalVariable(end_slot));
        self.instructions.push(Instruction::LessThanOrEqual);

        // Jump to end if expression is false
        let jump_not_true = self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("no jump-not-true provided")));

        // Compile statements inside loop block
        self.compile_statements(block);

        // compile step and jump back to start of loop
        self.instructions.push(Instruction::LoadLocalVariable(var_slot));
        self.instructions.push(Instruction::LoadLocalVariable(step_slot));
        self.instructions.push(Instruction::Add);
        self.instructions.push(Instruction::MoveToLocalVariable(var_slot));
        self.instructions.push(Instruction::JumpBackward(self.instructions.len() - start_of_loop));

        // Update jump not true value
        let jump_to_pos = self.instructions.len() - jump_not_true;
        self.instructions[jump_not_true] = Instruction::JumpIfFalse(jump_to_pos as i32);

    }

    // compile while loop
    fn compile_while_loop(&mut self, expr: Box<Token>, block: Vec<Token>) {
        trace!("compiling while loop");

        // Mark instruction pointer
        let start_ins_ptr = self.instructions.len();

        // Compile expression
        self.compile_expression(expr);

        // Jump to end if expression is false
        let jump_not_true = self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("no jump-not-true provided")));

        // Compile statements inside loop block
        self.compile_statements(block);

        // Goto loop start
        self.instructions.push(Instruction::JumpBackward(self.instructions.len() - start_ins_ptr));

        // Update jump not true value
        let jump_to_pos = self.instructions.len() - jump_not_true;
        self.instructions[jump_not_true] = Instruction::JumpIfFalse(jump_to_pos as i32);

    }


    //==============================================================================================
    // EXPRESSIONS

    // compile expression
    fn compile_expression(&mut self, token: Box<Token>) {

        match *token {

            Token::Null => {
                self.instructions.push(Instruction::StackPush(Value::Null));
            }

            Token::Integer(v) => {
                self.instructions.push(Instruction::StackPush(Value::Integer(v)));
            }

            Token::Float(v) => {
                self.instructions.push(Instruction::StackPush(Value::Float(v)));
            }

            Token::Bool(v) => {
                self.instructions.push(Instruction::StackPush(Value::Bool(v)));
            }

            Token::String(v) => {
                self.instructions.push(Instruction::StackPush(Value::String(v.to_string())));
            }

            Token::Identifier(ident) => {
                self.instructions.push(Instruction::LoadLocalVariable(self.get_variable(ident.as_str())));
            }

            Token::Array(elements) => {

                // Create empty array
                let ref_array = Rc::new(RefCell::new(Vec::default()));
                self.instructions.push(Instruction::StackPush(Value::Array(ref_array)));

                for element in elements {
                    self.compile_expression(Box::new(element));
                    self.instructions.push(Instruction::ArrayAdd);
                }

            }

            Token::Dictionary(pairs) => {

                // Create empty array
                let ref_hashmap = Rc::new(RefCell::new(HashMap::default()));
                self.instructions.push(Instruction::StackPush(Value::Dictionary(ref_hashmap)));

                for pair in pairs {
                    if let Token::KeyValuePair(k, value) = pair {
                        self.instructions.push(Instruction::StackPush(Value::String(k.to_string())));
                        self.compile_expression(value);
                        self.instructions.push(Instruction::DictionaryAdd);
                    }
                }

            }

            // Token::Object(class_name, params) => self.compile_new_object(class_name.to_string(), params),

            Token::CollectionIndex(id, index) => {
                trace!("i = {:?}, e = {:?}", id, index);

                // load array
                let Token::Identifier(id_name) = *id else { panic!("expected identifier") };
                self.instructions.push(Instruction::LoadLocalVariable(self.get_variable(id_name.as_str())));

                // compile index
                self.compile_expression(index);

                // get array value
                self.instructions.push(Instruction::GetCollectionItemByKey);

            }

            Token::Call(name, args) => {
                trace!("call = {:?}, args = {:?}", name, args);
                self.compile_call(name, args);
            }

            Token::Eq(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Equal);
            }

            Token::Ne(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::NotEqual);
            }

            Token::Add(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Add);
            }

            Token::Sub(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Sub);
            }

            Token::Mul(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Multiply);
            }

            Token::Div(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Divide);
            }

            Token::Pow(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Pow);
            }

            Token::Lt(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.instructions.push(Instruction::LessThan);
            }

            Token::Le(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.instructions.push(Instruction::LessThanOrEqual);
            }

            Token::Gt(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.instructions.push(Instruction::GreaterThan);
            }

            Token::Ge(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.instructions.push(Instruction::GreaterThanOrEqual);
            }

            // handle unreadable token and print what it is
            _ => panic!("unhandled token: {:?}", token),

        }
    }

    //==============================================================================================
    // HELPER FUNCTIONS

    // get index of variable or error if it doesn't exist
    fn get_variable(&self, name: &str) -> usize {
        if let Some(id) = self.variables.get(name) {
            *id
        } else {
            panic!("variable '{}' does not exist", name);
        }
    }

    // add variable and return its index or error if it already exists
    fn add_variable(&mut self, name: String) -> usize {

        // check if variable already exists
        if self.variables.contains_key(name.as_str()) {
            panic!("variable '{}' already exists", name);
        }

        // create variable
        let vid = self.variables.len();

        // add variable to list
        self.variables.insert(name.clone(), vid);

        vid
    }
}