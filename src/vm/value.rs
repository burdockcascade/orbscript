use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Not, Sub};
use std::rc::Rc;

// Value
#[derive(Clone, PartialEq, Debug)]
pub enum Value {

    // Values
    Null,
    Integer(i32),
    Pointer(usize),
    Float(f32),
    Bool(bool),
    String(String),
    Array(Rc<RefCell<Vec<Value>>>),
    Dictionary(Rc<RefCell<HashMap<String, Value>>>),
    Class(HashMap<String, Value>),
    Object(Rc<RefCell<HashMap<String, Value>>>),

    // counter with start, step, end
    Counter(i32, i32, i32),

    GlobalRef(String),
    FunctionPointer(usize),
    InstructionPointer(usize),

}

// function for finding Value by parameter. if its a number then return integer, if its a string then return string, etc.
impl Value {

    pub fn parse(param: &str) -> Value {
        match param.parse::<i32>() {
            Ok(num) => Value::Integer(num),
            Err(_) => match param.parse::<f32>() {
                Ok(num) => Value::Float(num),
                Err(_) => match param.parse::<bool>() {
                    Ok(b) => Value::Bool(b),
                    Err(_) => Value::String(param.to_string())
                }
            }
        }
    }

}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Integer(num) => write!(f, "{num}"),
            Value::Float(num) => write!(f, "{num}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(string) => write!(f, "{string}"),
            Value::Array(_val) => write!(f, "Array"),
            Value::FunctionPointer(name) => write!(f, "{name}"),
            _ => write!(f, "todo for {:?}", self),
        }
    }
}

// Value Comparison
impl PartialOrd for Value {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match (self, rhs) {
            (Value::Integer(v1), Value::Integer(v2)) => v1.partial_cmp(v2),
            (Value::Float(v1), Value::Float(v2)) => v1.partial_cmp(v2),
            _ => unreachable!("can not subtract values")
        }
    }
}

// Value Subtraction
impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> <Self as Sub<Value>>::Output {
        match (self, rhs) {
            (Value::Integer(v1), Value::Integer(v2)) => Value::Integer(v1 - v2),
            (Value::Integer(v1), Value::Float(v2)) => Value::Float(v1 as f32 - v2),
            (Value::Float(v1), Value::Integer(v2)) => Value::Float(v1 - v2 as f32),
            (Value::Float(v1), Value::Float(v2)) => Value::Float(v1 - v2),
            _ => unreachable!("can not subtract values")
        }
    }

}


// Value Addition
impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> <Self as Add<Value>>::Output {
        match (self, rhs) {

            // add integers together
            (Value::Integer(v1), Value::Integer(v2)) => Value::Integer(v1 + v2),
            (Value::Integer(v1), Value::Float(v2)) => Value::Float(v1 as f32 + v2),
            (Value::Integer(v1), Value::String(v2)) => Value::String(v1.to_string().add(v2.as_str())),

            // add floats together
            (Value::Float(v1), Value::Integer(v2)) => Value::Float(v1 + v2 as f32),
            (Value::Float(v1), Value::Float(v2)) => Value::Float(v1 + v2),

            // add strings together
            (Value::String(v1), Value::String(v2))  => Value::String(v1.add(v2.as_str())),
            (Value::String(v1), Value::Bool(v2)) => Value::String(v1.add(v2.to_string().as_str())),
            (Value::String(v1), Value::Integer(v2)) => Value::String(v1.add(v2.to_string().as_str())),
            (Value::String(v1), Value::Float(v2)) => Value::String(v1.add(v2.to_string().as_str())),

            // add arrays together
            (Value::Array(v1), Value::Array(v2)) => {
                v1.borrow_mut().extend(v2.borrow().iter().cloned());
                Value::Array(v1)
            },

            // add booleans together but only true + true = true
            (Value::Bool(v1), Value::Bool(v2)) => Value::Bool(v1 && v2),

            _ => unreachable!("can not add values")
        }
    }
}

// Value Multiplication
impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> <Self as Mul<Value>>::Output {
        match (self, rhs) {
            (Value::Integer(v1), Value::Integer(v2)) => Value::Integer(v1 * v2),
            (Value::Integer(v1), Value::Float(v2)) => Value::Float(v1 as f32 * v2),
            (Value::Float(v1), Value::Integer(v2)) => Value::Float(v1 * v2 as f32),
            (Value::Float(v1), Value::Float(v2)) => Value::Float(v1 * v2),
            _ => unreachable!("can not multiply values")
        }
    }
}

// Value Division
impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> <Self as Div<Value>>::Output {
        match (self, rhs) {
            (Value::Integer(v1), Value::Integer(v2)) => Value::Integer(v1 / v2),
            (Value::Integer(v1), Value::Float(v2)) => Value::Float(v1 as f32 / v2),
            (Value::Float(v1), Value::Integer(v2)) => Value::Float(v1 / v2 as f32),
            (Value::Float(v1), Value::Float(v2)) => Value::Float(v1 / v2),
            _ => unreachable!("can not divide values")
        }
    }
}

// Value Negation
impl Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        match self {
            Value::Bool(true) => Value::Bool(false),
            Value::Bool(false) => Value::Bool(true),
            _ => Value::Bool(false),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::vm::value::Value;

    #[test]
    fn test_add() {

        // integers
        assert_eq!(Value::Integer(2) + Value::Integer(3), Value::Integer(5));
        assert_eq!(Value::Integer(2) + Value::Float(3.3), Value::Float(5.3));

        // floats
        assert_eq!(Value::Float(2.2) + Value::Float(3.3), Value::Float(5.5));
        assert_eq!(Value::Float(2.2) + Value::Integer(3), Value::Float(5.2));

        // strings
        assert_eq!(Value::String(String::from("x = ")) + Value::Integer(3), Value::String(String::from("x = 3")));
        assert_eq!(Value::String(String::from("x = ")) + Value::Float(3.1), Value::String(String::from("x = 3.1")));
        assert_eq!(Value::String(String::from("x = ")) + Value::Bool(true), Value::String(String::from("x = true")));

        // true and false booleans should return false
        assert_eq!(Value::Bool(true) + Value::Bool(false), Value::Bool(false));
        assert_eq!(Value::Bool(false) + Value::Bool(true), Value::Bool(false));
        assert_eq!(Value::Bool(false) + Value::Bool(false), Value::Bool(false));
        assert_eq!(Value::Bool(true) + Value::Bool(true), Value::Bool(true));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Value::Integer(7) - Value::Integer(3), Value::Integer(4));
        assert_eq!(Value::Integer(5) - Value::Float(3.3), Value::Float(1.7));
        assert_eq!(Value::Float(2.4) - Value::Float(1.3), Value::Float(1.1000001));
        assert_eq!(Value::Float(5.2) - Value::Integer(3), Value::Float(2.1999998));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Value::Integer(7) * Value::Integer(3), Value::Integer(21));
        assert_eq!(Value::Integer(5) * Value::Float(1.1), Value::Float(5.5));
        assert_eq!(Value::Float(2.4) * Value::Float(1.3), Value::Float(3.1200001));
        assert_eq!(Value::Float(5.2) *  Value::Integer(3), Value::Float(15.599999));
    }

    #[test]
    fn test_div() {
        assert_eq!(Value::Integer(21) / Value::Integer(3), Value::Integer(7));
        assert_eq!(Value::Integer(22) / Value::Float(1.1), Value::Float(20.0));
        assert_eq!(Value::Float(2.4) / Value::Float(1.3), Value::Float(1.84615396));
        assert_eq!(Value::Float(5.2) /  Value::Integer(3), Value::Float(1.7333332));
    }

    #[test]
    fn test_eq() {
        assert_eq!(Value::Integer(3) == Value::Integer(3), true);
        assert_eq!(Value::Integer(21) == Value::Integer(3), false);
        assert_eq!(Value::Float(2.0) == Value::Integer(2), false);
        assert_eq!(Value::Float(2.0) == Value::Float(2.0), true);
        assert_eq!(Value::Bool(true) == Value::Bool(true), true);
        assert_eq!(Value::Bool(false) != Value::Bool(true), true);
        assert_eq!(Value::String("hello world".parse().unwrap()) == Value::String("hello world".parse().unwrap()), true);
        assert_eq!(Value::String("hello world".parse().unwrap()) == Value::String("goodbye world".parse().unwrap()), false);
    }

    #[test]
    fn test_cmp() {
        assert_eq!(Value::Integer(6) > Value::Integer(3), true);
        assert_eq!(Value::Integer(6) < Value::Integer(30), true);
        assert_eq!(Value::Float(6.1) > Value::Float(3.5), true);
    }

}