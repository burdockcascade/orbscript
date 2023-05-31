use std::rc::Rc;
use log::trace;
use crate::vm::frame::Frame;
use crate::vm::instructions::Instruction;
use crate::vm::program::Program;
use crate::vm::value::Value;

pub mod program;
pub mod instructions;
pub mod value;
mod frame;
mod iterator;

pub(crate) struct VM {
    ip: usize,
    frames: Vec<Frame>,
}

impl VM {

    pub fn new() -> VM {
        VM {
            ip: 0,
            frames: vec![],
        }
    }

    pub fn execute(&mut self, program: Program, parameters: Option<Vec<Value>>, entrypoint: Option<String>) -> Result<Option<Value>, String> {

        // Set entrypoint or use default
        let entry = match entrypoint {
            Some(entry) => entry,
            None => String::from("main")
        };

        // Set instruction pointer to entrypoint
        if program.globals.contains_key(entry.as_str()) {
            if let Value::FunctionRef(i) = program.globals.get(entry.as_str()).expect("program globals should have key") {
                self.ip = *i
            } else {
                return Err(format!("No entrypoint found: {:?}", entry));
            }
        } else {
            return Err(format!("No entrypoint found: {:?}", entry));
        }

        // push new frame
        self.frames.push(Frame::new(None, parameters));

        // set current frame
        let mut frame = self.frames.last_mut().expect("frame should be on the stack");

        loop {

            // get instruction
            let instruction = program.instructions.get(self.ip).expect(&*format!("instruction #{} should exist", self.ip));

            match instruction {

                //==================================================================================
                // BUILTINS

                Instruction::Assert => {

                    let output = frame.pop_value_from_stack();

                    match output {
                        Value::Bool(val) => assert!(val),
                        _ => panic!("unable to assert {}", output)
                    }

                    self.ip += 1;
                }

                Instruction::Print => {
                    let output = frame.pop_value_from_stack();
                    println!("{:?}", output.to_string());
                    self.ip += 1;
                }


                //==================================================================================
                // STACK

                Instruction::StackPush(value) => {
                    frame.push_value_to_stack(value.clone());
                    self.ip += 1;
                },


                //==================================================================================
                // CONTROL FLOW

                Instruction::JumpForward(delta) => {
                    self.ip += *delta as usize;
                }

                Instruction::JumpBackward(delta) => {
                    self.ip -= *delta as usize;
                }

                Instruction::JumpIfFalse(delta) => {

                    let b = frame.pop_value_from_stack();

                    match b {
                        Value::Bool(false) =>{
                            if *delta > 0 {
                                self.ip += *delta as usize;
                            } else {
                                self.ip -= *delta as usize;
                            }
                        },
                        _ => self.ip += 1
                    }
                }


                //==================================================================================
                // VARIABLES

                // get value from stack and store in variable
                Instruction::MoveToLocalVariable(index) => {
                    frame.move_from_stack_to_variable_slot(*index);
                    self.ip += 1;
                }

                Instruction::CopyToLocalVariable(index) => {
                    frame.copy_from_stack_to_variable_slot(*index);
                    self.ip += 1;
                }

                // get value from variable and push onto stack
                Instruction::LoadLocalVariable(index) => {
                    frame.copy_from_variable_slot_to_stack(*index);
                    self.ip += 1;
                }

                Instruction::LoadGlobal(name) => {
                    let function_ref = program.globals.get(name).expect(&*format!("global variable {:?} should exist", name));
                    frame.push_value_to_stack(function_ref.clone());
                    self.ip += 1;
                },

                //==================================================================================
                // FUNCTIONS

                Instruction::Call(arg_len) => {

                    // cut args from stack and then reverse order
                    let mut args = frame.pop_values_from_stack(*arg_len as usize);
                    args.reverse();

                    // pop functionref from stack
                    let Value::FunctionRef(function_position) = frame.pop_value_from_stack() else {
                        return Err(format!("functionRef should be on the stack"));
                    };

                    let a = if args.is_empty() {
                        None
                    } else {
                        Some(args)
                    };

                    // push new frame onto frames
                    let next_ip = self.ip + 1;
                    self.frames.push(Frame::new(Some(next_ip), a));

                    // set current frame
                    frame = self.frames.last_mut().expect("frame should be on the stack");

                    // set instruction pointer to function
                    self.ip = function_position;

                }

                Instruction::Return(has_return_value) => {

                    // pop return value from stack
                    let return_value = if *has_return_value {
                        frame.pop_value_from_stack()
                    } else {
                        Value::Null
                    };

                    // if no return position, then we are at the end of the program
                    if frame.return_position == None {
                        return Ok(None);
                    }

                    // set instruction back to previous location
                    self.ip = frame.return_position.expect("return position should be set");

                    // remove last frame
                    self.frames.pop();

                    // set new current frame
                    frame = self.frames.last_mut().expect("frame should be on the stack");

                    // push return value onto stack
                    if *has_return_value {
                        frame.push_value_to_stack(return_value);
                    }

                }

                
                //==================================================================================
                // ARRAYS

                // get array length
                Instruction::ArrayLength => {

                    let array = frame.pop_value_from_stack();

                    if let Value::Array(val) = array {
                        frame.push_value_to_stack(Value::Integer(val.borrow().len() as i32));
                    } else {
                        panic!("can not get length on non-array {}", array)
                    }

                    self.ip += 1;

                }

                // add value to array
                Instruction::ArrayAdd => {

                    let value = frame.pop_value_from_stack();
                    let array = frame.pop_value_from_stack();

                    if let Value::Array(v) = array {
                        v.borrow_mut().push(value);
                        frame.push_value_to_stack(Value::Array(v));
                    }

                    self.ip += 1;
                }
                

                //==================================================================================
                // DICTIONARY

                Instruction::DictionaryAdd => {
                    let value = frame.pop_value_from_stack();
                    let key = frame.pop_value_from_stack();
                    let dict = frame.pop_value_from_stack();

                    if let Value::Dictionary(v) = dict {
                        v.borrow_mut().insert(key.to_string(), value);
                        frame.push_value_to_stack(Value::Dictionary(v));
                    }

                    self.ip += 1;
                }
                

                //==================================================================================
                // KEY VALUE

                Instruction::GetCollectionItemByKey | Instruction::ArrayGet => {

                    let key = frame.pop_value_from_stack();
                    let collection = frame.pop_value_from_stack();

                    match collection {

                        Value::Array(items) => {

                            if let Value::Integer(index) = key {
                                let borrowed_items = items.borrow();
                                let array_value = borrowed_items.get(index as usize).expect(format!("array index {} should exist", index).as_str());
                                frame.push_value_to_stack(array_value.clone());
                            } else {
                                panic!("can not get index on non-integer {}", key)
                            }
                        },

                        Value::Dictionary(items) => {

                            if let Value::String(index) = key {
                                let items_borrowed = items.borrow();
                                let v2 = items_borrowed.get(index.as_str()).expect(&*format!("key '{}' should exist in dictionary", index));
                                frame.push_value_to_stack(v2.clone());
                            } else {
                                panic!("can not get index on non-string {}", key)
                            }
                        }

                        _ => panic!("can not get index on non-collection {}", key)

                    }

                    self.ip += 1;
                }

                Instruction::SetCollectionItemByKey => {

                    let key = frame.pop_value_from_stack();
                    let value = frame.pop_value_from_stack();
                    let collection = frame.pop_value_from_stack();

                    match collection {
                        Value::Array(items) => {
                            if let Value::Integer(index) = key {
                                items.borrow_mut()[index as usize] = value;
                                frame.push_value_to_stack(Value::Array(items));
                            } else {
                                panic!("can not get index on non-integer {}", key)
                            }
                        },
                        Value::Dictionary(items) => {
                            if let Value::String(index) = key {
                                items.borrow_mut().insert(index, value);
                                frame.push_value_to_stack(Value::Dictionary(items));
                            } else {
                                panic!("can not get index on non-string {}", key)
                            }
                        }
                        _ => panic!("can not get index on non-collection")
                    }

                    self.ip += 1;
                }



                //==================================================================================
                // ARITHMETIC

                Instruction::Add => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs + rhs);
                    self.ip += 1;
                }

                Instruction::Sub => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs - rhs);
                    self.ip += 1;
                }

                Instruction::Multiply => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs * rhs);
                    self.ip += 1;
                }

                Instruction::Divide => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs / rhs);
                    self.ip += 1;
                }

                Instruction::Pow => {
                    // todo: implement
                    self.ip += 1;
                }

                //==================================================================================
                // OPERANDS

                Instruction::Equal => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs == rhs));
                    self.ip += 1;
                }

                Instruction::NotEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs != rhs));
                    self.ip += 1;
                }

                Instruction::LessThan => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs < rhs));
                    self.ip += 1;
                }

                Instruction::LessThanOrEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs <= rhs));
                    self.ip += 1;
                }

                Instruction::GreaterThan => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs > rhs));
                    self.ip += 1;
                }

                Instruction::GreaterThanOrEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs >= rhs));
                    self.ip += 1;
                }

                _ => unimplemented!("instruction {:?}", instruction)
            }

        }

    }

}