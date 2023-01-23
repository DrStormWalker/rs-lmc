mod compiler;
mod error;
mod instruction;
mod interpreter;
mod memory;
mod span;

use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use compiler::compile_lmc_asm;
use error::{CompilerError, CompilerResult};
use interpreter::{InterpreterError, Vm};
use span::SourceBuffer;
use thiserror::Error;

use crate::memory::{DynamicMemoryStorage, MemoryStorage};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct RunArgs {
    filepath: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run(RunArgs),
}

#[derive(Debug, Error)]
pub enum RunError<'a> {
    #[error(transparent)]
    CompilerError(CompilerError<'a>),
    #[error(transparent)]
    InterpreterError(#[from] InterpreterError),
}

pub type RunResult<'a, T> = Result<T, RunError<'a>>;

fn run<'a>(args: &RunArgs, asm: &'a str) -> RunResult<'a, ()> {
    let source = SourceBuffer::new(&asm);

    let insts = compile_lmc_asm(&source.source()).map_err(|e| RunError::CompilerError(e))?;

    let mut memory = DynamicMemoryStorage::<String>::new();

    for (i, inst) in insts.into_iter().enumerate() {
        memory
            .set_inst(i, inst)
            .map_err(|e| InterpreterError::MemoryError(e))?;
    }

    let mut vm = Vm::new(memory);

    vm.run_program()?;

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let args = Cli::parse();

    match &args.command {
        Commands::Run(run_args) => {
            let mut asm = String::new();
            File::open(&run_args.filepath)
                .unwrap()
                .read_to_string(&mut asm)?;

            if let Err(e) = run(run_args, &asm) {
                println!("\n{}", e);
            }
        }
    }

    Ok(())
}
