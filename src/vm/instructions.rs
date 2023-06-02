use crate::vm::value::Value;

// Instruction
#[derive(Clone, Debug)]
pub enum Instruction {

    // Stack
    PushNull,
    PushInteger(i32),
    PushFloat(f32),
    PushBool(bool),
    PushString(String),

    // Variables
    MoveToLocalVariable(usize),
    LoadLocalVariable(usize),

    // Global
    LoadGlobal(String),

    // Objects
    CreateObject,

    // Collections
    GetCollectionItem,
    SetCollectionItem,
    CreateCollectionAsDictionary(usize),
    CreateCollectionAsArray(usize),

    // Iteration
    IteratorStart,
    IteratorNext(usize, usize),
    
    // Instructions
    Call(usize),
    JumpForward(usize),
    JumpBackward(usize),
    JumpIfFalse(i32),
    Return(bool),

    // Operators
    Equal,
    NotEqual,
    Add,
    Sub,
    Multiply,
    Divide,
    Pow,

    // Comparison
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    // Halt Program
    Halt(String)

}