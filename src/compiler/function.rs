use std::collections::HashMap;
use crate::compiler::token::Token;
use crate::vm::instructions::Instruction;

#[derive(Clone)]
pub struct Function {
    pub instructions: Vec<Instruction>,
    variables: HashMap<String, usize>,
    pub anon_functions: HashMap<String, Vec<Instruction>>
}

impl Function {
    pub fn new(parameters: Vec<Token>, body: Vec<Token>) -> Function {

        let mut f = Function {
            instructions: vec![],
            variables: Default::default(),
            anon_functions: Default::default()
        };

        // store the parameters as variables
        f.add_parameters(parameters.clone());

        // compile the statements
        f.compile_statements(body);

        // if tha last instruction is not a return then add one
        if matches!(f.instructions.last(), Some(Instruction::Return(_))) == false {
            f.instructions.push(Instruction::Return(false));
        }

        f
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
            Token::Variable(name, expr) => self.compile_variable(name, expr),
            Token::Assign(name, expr) => self.compile_assignment(name, expr),
            Token::Call(name, args) => self.compile_call(name, args),
            Token::Return(expr) => self.compile_return(expr),
            Token::WhileLoop(cond, body) => self.compile_while_loop(cond, body),
            Token::ForI(var, start, step, end, body) => self.compile_iterator(var, start, step, end, body),
            Token::ForEach(var, collection, body) =>   self.compile_iterator(var, Box::new(Token::Integer(0)),  Box::new(Token::Integer(1)), collection, body),
            Token::IfElse(cond, body, else_body) => self.compile_if_else(cond, body, else_body),
            Token::Comment(_) => { },
            _ => unimplemented!("statement not implemented: {:?}", statement)
        }
    }


    //==============================================================================================
    // VARIABLES

    fn add_parameters(&mut self, parameters: Vec<Token>) {
        for param in parameters {
            self.add_variable(param.to_string());
        }
    }

    fn compile_variable(&mut self, name: Box<Token>, expr: Box<Token>) {

        // Declare variable
        let slot = self.add_variable(name.to_string());

        // compile the value
        self.compile_expression(expr);

        // store the value
        self.instructions.push(Instruction::MoveToLocalVariable(slot));

    }

    // compile assignment
    fn compile_assignment(&mut self, left: Box<Token>, right: Box<Token>) {

        match *left.clone() {

            // store value in variable
            Token::Identifier(name) => {

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

                // load the variable
                let Token::Identifier(name_as_string) = *name else { panic!("name is not an identifier") };
                let slot = self.get_variable(name_as_string.as_str());
                self.instructions.push(Instruction::LoadLocalVariable(slot));

                // compile the value
                self.compile_expression(right);

                // compile the index
                self.compile_expression(index);

                // add value to collection
                self.instructions.push(Instruction::SetCollectionItem);

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

        if self.variables.contains_key(&function_name) {
            self.instructions.push(Instruction::LoadLocalVariable(self.get_variable(function_name.as_str())));
        } else {
            self.instructions.push(Instruction::PushString(function_name));
        }

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

    fn compile_iterator(&mut self, var: Box<Token>, counter_start_at: Box<Token>, counter_step: Box<Token>, target: Box<Token>,  block: Vec<Token>) {

        // compile var
        let var_slot = self.add_variable(var.to_string());

        // compile target
        self.compile_expression(target);

        // compile counter step
        self.compile_expression(counter_step);

        // compile counter start
        self.compile_expression(counter_start_at);

        // Create Iterator
        self.instructions.push(Instruction::IteratorStart);

        // temp jump to end
        let start_ins_ptr = self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("iterator not updated")));

        // compile statements inside loop block
        self.compile_statements(block);

        // jump back to start
        self.instructions.push(Instruction::JumpBackward(self.instructions.len() - start_ins_ptr));

        // update iterator
        let jump_to_pos = self.instructions.len() - start_ins_ptr;
        self.instructions[start_ins_ptr] = Instruction::IteratorNext(var_slot, jump_to_pos);

    }

    // compile while loop
    fn compile_while_loop(&mut self, expr: Box<Token>, block: Vec<Token>) {

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
                self.instructions.push(Instruction::PushNull);
            }

            Token::Integer(v) => {
                self.instructions.push(Instruction::PushInteger(v));
            }

            Token::Float(v) => {
                self.instructions.push(Instruction::PushFloat(v));
            }

            Token::Bool(v) => {
                self.instructions.push(Instruction::PushBool(v));
            }

            Token::String(v) => {
                self.instructions.push(Instruction::PushString(v));
            }

            Token::Identifier(ident) => {
                if self.variables.contains_key(ident.as_str()) {
                    self.instructions.push(Instruction::LoadLocalVariable(self.get_variable(ident.as_str())));
                } else {
                    self.instructions.push(Instruction::LoadGlobal(ident));
                }
            }

            Token::Array(elements) => {

                let array_size = elements.len();

                // Compile each element
                for element in elements {
                    self.compile_expression(Box::new(element));
                }

                // collect items into array
                self.instructions.push(Instruction::CreateCollectionAsArray(array_size));

            }

            Token::Dictionary(pairs) => {

                let dict_size = pairs.len();

                for pair in pairs {
                    if let Token::KeyValuePair(k, value) = pair {
                        self.instructions.push(Instruction::PushString(k));
                        self.compile_expression(value);
                    } else {
                        panic!("expected key value pair");
                    }
                }

                // collect items into dictionary
                self.instructions.push(Instruction::CreateCollectionAsDictionary(dict_size));

            }

            Token::AnonFunction(args, body) => {

                // create a new function
                let func_name = format!("lambda_{}", self.anon_functions.len());
                let f = Function::new(args, body);

                self.anon_functions.insert(func_name.clone(), f.instructions);

                // push globalref onto stack
                self.instructions.push(Instruction::PushString(func_name));
            }

            // Token::Object(class_name, params) => self.compile_new_object(class_name.to_string(), params),

            Token::CollectionIndex(id, index) => {

                // load array
                let Token::Identifier(id_name) = *id else { panic!("expected identifier") };
                self.instructions.push(Instruction::LoadLocalVariable(self.get_variable(id_name.as_str())));

                // compile index
                self.compile_expression(index);

                // get array value
                self.instructions.push(Instruction::GetCollectionItem);

            }

            Token::Call(name, args) => {
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