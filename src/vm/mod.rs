use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use log::debug;
use crate::vm::frame::Frame;
use crate::vm::instructions::Instruction;
use crate::vm::program::Program;
use crate::vm::value::Value;

pub mod program;
pub mod instructions;
pub mod value;
mod frame;

pub(crate) struct VM {
}

impl VM {

    pub fn new() -> VM {
        VM {

        }
    }

    pub fn execute(&mut self, program: Program, parameters: Option<Vec<Value>>, entrypoint: Option<String>) -> Result<Option<Value>, String> {

        let mut ip: usize;
        let mut frames: Vec<Frame> = vec![];
        
        // Set entrypoint or use default
        let entry = match entrypoint {
            Some(entry) => entry,
            None => String::from("main")
        };

        // Set instruction pointer to entrypoint
        if program.globals.contains_key(entry.as_str()) {
            if let Value::FunctionPointer(i) = program.globals.get(entry.as_str()).expect("program globals should have key") {
                ip = *i
            } else {
                return Err(format!("No entrypoint found: {:?}", entry));
            }
        } else {
            return Err(format!("No entrypoint found: {:?}", entry));
        }

        // push new frame
        frames.push(Frame::new(None, parameters));

        // set current frame
        let mut frame = frames.last_mut().expect("frame should be on the stack");
        loop {

            // get instruction
            let instruction = program.instructions.get(ip).expect(&*format!("instruction #{} should exist", ip));

            // debug!("ip: {}, instruction: {:?}", ip, instruction);
            // debug!("variables: {:?}", frame.variables);
            // debug!("stack: {:?}", frame.data);

            match instruction {

                //==================================================================================
                // BUILTINS

                Instruction::Assert => {

                    let output = frame.pop_value_from_stack();

                    match output {
                        Value::Bool(val) => assert!(val),
                        _ => panic!("unable to assert {}", output)
                    }

                    ip += 1;
                }

                Instruction::Print => {
                    let output = frame.pop_value_from_stack();
                    println!("{:?}", output.to_string());
                    ip += 1;
                }


                //==================================================================================
                // STACK

                Instruction::PushNull => {
                    frame.push_value_to_stack(Value::Null);
                    ip += 1;
                }

                Instruction::PushInteger(value) => {
                    frame.push_value_to_stack(Value::Integer(*value));
                    ip += 1;
                }

                Instruction::PushFloat(value) => {
                    frame.push_value_to_stack(Value::Float(*value));
                    ip += 1;
                }

                Instruction::PushBool(value) => {
                    frame.push_value_to_stack(Value::Bool(*value));
                    ip += 1;
                }

                Instruction::PushString(value) => {
                    frame.push_value_to_stack(Value::String(value.clone()));
                    ip += 1;
                }

                Instruction::PushGlobalRef(name) => {
                    let value = program.globals.get(name).expect(&*format!("global variable {:?} should exist", name));
                    frame.push_value_to_stack(value.clone());
                    ip += 1;
                }


                //==================================================================================
                // CONTROL FLOW

                Instruction::JumpForward(delta) => {
                    ip += *delta as usize;
                }

                Instruction::JumpBackward(delta) => {
                    ip -= *delta as usize;
                }

                Instruction::JumpIfFalse(delta) => {

                    let b = frame.pop_value_from_stack();

                    match b {
                        Value::Bool(false) =>{
                            if *delta > 0 {
                                ip += *delta as usize;
                            } else {
                                ip -= *delta as usize;
                            }
                        },
                        _ => ip += 1
                    }
                }


                //==================================================================================
                // VARIABLES

                // get value from stack and store in variable
                Instruction::MoveToLocalVariable(index) => {
                    frame.move_from_stack_to_variable_slot(*index);
                    ip += 1;
                }

                Instruction::CopyToLocalVariable(index) => {
                    frame.copy_from_stack_to_variable_slot(*index);
                    ip += 1;
                }

                // get value from variable and push onto stack
                Instruction::LoadLocalVariable(index) => {
                    frame.copy_from_variable_slot_to_stack(*index);
                    ip += 1;
                }

                Instruction::LoadGlobal(name) => {
                    let function_ref = program.globals.get(name).expect(&*format!("global variable {:?} should exist", name));
                    frame.push_value_to_stack(function_ref.clone());
                    ip += 1;
                },

                //==================================================================================
                // FUNCTIONS

                Instruction::Call(arg_len) => {

                    // cut args from stack and then reverse order
                    let mut args = frame.pop_values_from_stack(*arg_len as usize);
                    args.reverse();

                    // get function position
                    let function_position = match frame.pop_value_from_stack() {
                        Value::FunctionPointer(function_position) => function_position,
                        Value::GlobalRef(global_ref) => {
                            if let Value::FunctionPointer(function_position) = program.globals.get(global_ref.as_str()).expect("global should exist") {
                                *function_position
                            } else {
                                panic!("global should be a functionref");
                            }
                        },
                        _ => panic!("functionref should be on the stack")
                    };

                    let a = if args.is_empty() {
                        None
                    } else {
                        Some(args)
                    };

                    // push new frame onto frames
                    let next_ip = ip + 1;
                    frames.push(Frame::new(Some(next_ip), a));

                    // set current frame
                    frame = frames.last_mut().expect("frame should be on the stack");

                    // set instruction pointer to function
                    ip = function_position;

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
                    ip = frame.return_position.expect("return position should be set");

                    // remove last frame
                    frames.pop();

                    // set new current frame
                    frame = frames.last_mut().expect("frame should be on the stack");

                    // push return value onto stack
                    if *has_return_value {
                        frame.push_value_to_stack(return_value);
                    }

                }


                //==================================================================================
                // COLLECTIONS

                Instruction::CreateCollectionAsDictionary(size) => {

                    let mut items = HashMap::new();

                    for _ in 0..*size {

                        let value = frame.pop_value_from_stack();
                        let key = frame.pop_value_from_stack();

                        match key {
                            Value::String(key) => {
                                items.insert(key, value);
                            },
                            _ => panic!("can not create dictionary with non-string key {}", key)
                        }
                    }

                    frame.push_value_to_stack(Value::Dictionary(Rc::new(RefCell::new(items))));

                    ip += 1;
                }

                Instruction::CreateCollectionAsArray(size) => {

                    let mut items = Vec::new();

                    for _ in 0..*size {
                        let value = frame.pop_value_from_stack();
                        items.push(value);
                    }

                    items.reverse();

                    frame.push_value_to_stack(Value::Array(Rc::new(RefCell::new(items))));

                    ip += 1;
                }

                Instruction::GetCollectionItem => {

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
                        _ => panic!("can not get index on non-collection {}", collection)
                    }

                    ip += 1;
                }

                Instruction::SetCollectionItem => {

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
                        _ => panic!("can not get index on non-collection {}", collection)
                    }

                    ip += 1;
                }


                //==================================================================================
                // ITERATION

                Instruction::IteratorStart => {

                    let Value::Integer(start) = frame.pop_value_from_stack() else {
                        panic!("start should be an integer");
                    };

                    let Value::Integer(step) = frame.pop_value_from_stack() else {
                        panic!("step should be an integer");
                    };

                    let target = frame.pop_value_from_stack();

                    let end = match target {
                        Value::Integer(i) => {
                            frame.push_value_to_stack(target);
                            i
                        },
                        Value::Array(items) => {
                            frame.push_value_to_stack(Value::Array(items.clone()));
                            items.borrow().len() as i32 - 1
                        },
                        Value::Dictionary(items) => {

                            // get dictionary keys and map to value string
                            let keys = items.borrow().keys().map(|k| Value::String(k.clone())).collect::<Vec<Value>>();

                            // get keys length
                            let keys_length = keys.len() as i32 - 1;

                            // push keys onto stack
                            frame.push_value_to_stack(Value::Array(Rc::new(RefCell::new(keys))));

                            keys_length
                        },
                        _ => panic!("can not iterate over non-integer, array or dictionary")
                    };

                    // push counter onto stack
                    frame.push_value_to_stack(Value::Counter(start, step, end));

                    ip += 1;
                }

                Instruction::IteratorNext(var_slot, ip_delta) => {

                    let Value::Counter(index, step, end) = frame.pop_value_from_stack() else {
                        panic!("invalid counter on stack");
                    };

                    match frame.pop_value_from_stack() {

                        Value::Integer(i) => {

                            // calculate next count
                            let next_count = index + step;

                            // if next count is greater than i, then skip to next instruction
                            if next_count > i + 1 {
                                ip += ip_delta;
                                continue;
                            }

                            // push value to variable slot
                            frame.push_value_to_variable_slot(*var_slot, Value::Integer(index as i32));

                            // push ite and counter back onto stack
                            frame.push_value_to_stack(Value::Integer(i));
                            frame.push_value_to_stack(Value::Counter(next_count, step, end));

                        }
                        Value::Array(items) => {

                            // calculate next count
                            let next_count = index + step;

                            if next_count > end + 1 {
                                ip += ip_delta;
                                continue;
                            }

                            // get item from array
                            let borrowed_items = items.borrow();
                            let array_value = borrowed_items.get(index as usize).expect(format!("array index {} should exist", index).as_str());

                            // push value to variable slot
                            frame.push_value_to_variable_slot(*var_slot, array_value.clone());

                            // push collection back onto stack
                            frame.push_value_to_stack(Value::Array(items.clone()));
                            frame.push_value_to_stack(Value::Counter(index + 1, step, end));

                        },
                        _ => panic!("can not iterate over this value type")
                    }

                    ip += 1;
                }


                //==================================================================================
                // ARITHMETIC

                Instruction::Add => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs + rhs);
                    ip += 1;
                }

                Instruction::Sub => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs - rhs);
                    ip += 1;
                }

                Instruction::Multiply => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs * rhs);
                    ip += 1;
                }

                Instruction::Divide => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs / rhs);
                    ip += 1;
                }

                Instruction::Pow => {
                    // todo: implement
                    ip += 1;
                }

                //==================================================================================
                // OPERANDS

                Instruction::Equal => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs == rhs));
                    ip += 1;
                }

                Instruction::NotEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs != rhs));
                    ip += 1;
                }

                Instruction::LessThan => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs < rhs));
                    ip += 1;
                }

                Instruction::LessThanOrEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs <= rhs));
                    ip += 1;
                }

                Instruction::GreaterThan => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs > rhs));
                    ip += 1;
                }

                Instruction::GreaterThanOrEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs >= rhs));
                    ip += 1;
                }

                _ => unimplemented!("instruction {:?}", instruction)
            }

        }


    }

}
