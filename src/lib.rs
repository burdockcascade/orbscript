use log::{debug, info, LevelFilter};
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use crate::compiler::Compiler;
use crate::vm::value::Value;
use crate::vm::VM;

mod compiler;
mod vm;

pub fn run(program: &str, parameters: Option<Vec<Value>>, entry: Option<String>) -> Result<Option<Value>, String> {

    let _ = TermLogger::init(LevelFilter::Trace, Config::default(),TerminalMode::Mixed, ColorChoice::Auto);

    info!("Running program");

    let mut c = Compiler::new();
    let p = c.compile(program)?;

    let mut vm = VM::new();

    // add callback to vm that prints helloworld
    vm.add_builtin_function("print", |values| {
        let v = values.get(0).expect("No value to print");
        println!("{:?}", v.to_string());
        None
    });

    vm.add_builtin_function("assertTrue", |mut values| {

        let msg = values.pop().expect("No msg provided");
        let bool = values.pop().expect("No boolean");

        if bool != Value::Bool(true) {
            panic!("Assertion failed: {}", msg.to_string());
        }

        None
    });

    vm.add_builtin_function("assertEquals", |mut values| {

        let msg = values.pop().expect("No msg provided");
        let v2 = values.pop().expect("No boolean");
        let v1 = values.pop().expect("No boolean");

        if v1 != v2 {
            panic!("Assertion failed: {}", msg.to_string());
        }

        None
    });

    vm.execute(p, parameters, entry)

}
