use std::cell::RefCell;
use std::rc::Rc;
use crate::vm::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct Iterator {
    index: usize,
    max: usize,
    array: Rc<RefCell<Vec<Value>>>,
}

