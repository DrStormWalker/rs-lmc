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
use compiler::compile_lmc_asm_with_source_map;
use error::{CompilerError, InterpreterErrorRenderer, SourceErrorRenderHelper};
use interpreter::{InterpreterError, Vm};
use memory::MemoryError;
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

fn run<'a>(args: &RunArgs, asm: &'a str) -> Option<()> {
    let source = SourceBuffer::new(&asm);

    let filepath = args.filepath.display().to_string();

    let (insts, source_map) = match compile_lmc_asm_with_source_map(&source.source()) {
        Ok((insts, source_map)) => (insts, source_map),
        Err(e) => {
            let render = e.render(&source, &filepath);

            println!("{}", render);

            return None;
        }
    };

    let mut memory = DynamicMemoryStorage::<String>::new();

    for (i, inst) in insts.into_iter().enumerate() {
        if let Err(e) = memory.set_inst(i, inst) {
            todo!();
        }
    }

    let mut vm = Vm::new(memory);

    if let Err(e) = vm.run_program() {
        let render = InterpreterErrorRenderer {
            error: e,
            source_map: &source_map,
            source: &source,
            filepath: &filepath,
        };

        println!("\n{}", render);

        return None;
    }

    Some(())
}

fn main() -> Result<(), io::Error> {
    let args = Cli::parse();

    match &args.command {
        Commands::Run(run_args) => {
            let mut asm = String::new();
            File::open(&run_args.filepath)
                .unwrap()
                .read_to_string(&mut asm)?;

            run(run_args, &asm);
        }
    }

    Ok(())
}
