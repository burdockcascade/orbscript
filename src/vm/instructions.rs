use crate::vm::value::Value;

// Instruction
#[derive(Clone, Debug)]
pub enum Instruction {

    // Built-in Functions
    Assert,
    Print,

    // Stack
    StackPush(Value),

    // Variables
    MoveToLocalVariable(usize),
    CopyToLocalVariable(usize),
    LoadLocalVariable(usize),

    // Global
    LoadGlobal(String),

    // Objects
    CreateObject,

    // Dictionaries
    DictionaryKeys,
    DictionaryAdd,
    DictionaryGet,
    DictionarySet,

    // Arrays
    ArrayLength,
    ArrayAdd,
    ArrayGet,
    ArraySet,

    // Key Value
    GetCollectionItemByKey,
    SetCollectionItemByKey,

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