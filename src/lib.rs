use log::{info, LevelFilter, trace};
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
    vm.execute(p, parameters, entry)

}
