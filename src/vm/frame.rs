use crate::vm::value::Value;

#[derive(Debug)]
pub struct Frame {
    pub return_position: Option<usize>,
    variables: Vec<Value>,
    data: Vec<Value>,
}

impl Frame {

    // new frame with parameter as name
    pub fn new(return_position: Option<usize>, args: Option<Vec<Value>>) -> Frame {

        Frame {
            return_position,
            variables: args.unwrap_or(vec![]),
            data: vec![],
        }
    }

    // push a value to the stack
    pub fn push_value_to_stack(&mut self, value: Value) {
        self.data.push(value);
    }

    // push a value to a variable slot
    pub fn push_value_to_variable_slot(&mut self, slot: usize, value: Value) {
        if self.variables.len() <= slot {
            self.variables.resize(slot + 1, value);
        } else {
            self.variables[slot] = value;
        }
    }

    // move a value from the stack to a variable slot
    pub fn move_from_stack_to_variable_slot(&mut self, slot: usize) {
        let value = self.pop_value_from_stack();
        self.push_value_to_variable_slot(slot, value);
    }

    // copy value from the stack to a variable slot
    pub fn copy_from_stack_to_variable_slot(&mut self, slot: usize) {
        let value = self.get_top_value_on_stack();
        self.push_value_to_variable_slot(slot, value);
    }

    // copy from variable slot to stack
    pub fn copy_from_variable_slot_to_stack(&mut self, slot: usize) {
        let value = self.get_variable_or_panic(slot).clone();
        self.push_value_to_stack(value);
    }

    // return a clone of the top value on the stack
    pub fn get_top_value_on_stack(&self) -> Value {
        let value = self.data.last().expect("stack should have a value");
        return value.clone();
    }

    // pop a value from the stack
    pub fn pop_value_from_stack(&mut self) -> Value {
        let value = self.data.pop().expect("stack should have a value");
        return value;
    }

    // pop 2 values from the stack
    pub fn pop_2_values_from_stack(&mut self) -> (Value, Value) {
        let rhs = self.pop_value_from_stack();
        let lhs = self.pop_value_from_stack();
        return (lhs, rhs);
    }

    // pop values from the stack
    pub fn pop_values_from_stack(&mut self, count: usize) -> Vec<Value> {
        let mut values = vec![];
        for _ in 0..count {
            values.push(self.pop_value_from_stack());
        }
        return values;
    }

    // get the value from the variable slot
    pub fn get_variable_or_panic(&self, slot: usize) -> &Value {
        self.variables.get(slot).expect("variable slot should exist")
    }

}