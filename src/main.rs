#![feature(exact_size_is_empty)]

mod assembler;
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

use assembler::assemble_lmc_asm;
use clap::{Args, Parser, Subcommand};
use error::InterpreterErrorRenderer;
use interpreter::Vm;
use span::SourceBuffer;

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

    #[arg(long, default_value_t = false)]
    print_expanded: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run(RunArgs),
}

fn run<'a>(args: &RunArgs, asm: &'a str) -> Option<()> {
    let source = SourceBuffer::new(&asm);

    let filepath = args.filepath.display().to_string();

    let (insts, source_map) = match assemble_lmc_asm(&source.source(), args.print_expanded) {
        Ok((insts, source_map)) => (insts, source_map),
        Err(e) => {
            for e in e {
                let render = e.render(&source, &filepath);

                println!("{}", render);
            }

            return None;
        }
    };

    let mut memory = DynamicMemoryStorage::<String>::new();

    for (i, inst) in insts.into_iter().enumerate() {
        if let Err(_) = memory.set_inst(i, inst) {
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

            let asm = asm.replace("\t", "    ");

            run(run_args, &asm);
        }
    }

    Ok(())
}
