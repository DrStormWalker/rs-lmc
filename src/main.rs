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

use assembler::SourceMap;
use clap::{Args, Parser, Subcommand};
use error::{CompilerResult, InterpreterErrorRenderer};
use instruction::Inst;
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
    no_macros: bool,
}

#[derive(Args, Debug)]
struct ExpandArgs {
    filepath: PathBuf,

    #[arg(long, default_value_t = false)]
    no_macros: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run(RunArgs),
    Expand(ExpandArgs),
}

fn _run<'a>(args: &RunArgs, asm: &'a str) -> CompilerResult<'a, (Vec<Inst>, SourceMap)> {
    use assembler::{assemble_lmc_asm, tokenize_and_parse_lmc_asm};

    let (insts, symbol_table) = tokenize_and_parse_lmc_asm(asm, !args.no_macros)?;

    assemble_lmc_asm(insts, symbol_table)
}

fn run<'a>(args: &RunArgs, asm: &'a str) -> Option<()> {
    let source = SourceBuffer::new(&asm);

    let filepath = args.filepath.display().to_string();

    let (insts, source_map) = match _run(args, source.source()) {
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

fn _expand<'a>(args: &ExpandArgs, asm: &'a str) -> CompilerResult<'a, ()> {
    use assembler::tokenize_and_parse_lmc_asm;
    use std::iter;

    let (insts, _) = tokenize_and_parse_lmc_asm(asm, !args.no_macros)?;

    let label_padding = insts
        .iter()
        .filter_map(|i| i.label.as_ref().map(|l| l.source.len()))
        .max()
        .map_or(0, |v| v + 2);

    let label_padding: String = iter::repeat(' ').take(label_padding).collect();

    for inst in insts {
        let label = inst
            .label
            .map(|l| l.source[..].to_string())
            .unwrap_or("".to_string());

        println!(
            "{}{}{}{}",
            label,
            &label_padding[label.len()..],
            inst.opcode.1.as_str(),
            inst.operand
                .map(|o| format!("  {}", o.1.to_string()))
                .unwrap_or("".to_string()),
        )
    }

    Ok(())
}

fn expand<'a>(args: &ExpandArgs, asm: &'a str) -> Option<()> {
    let source = SourceBuffer::new(&asm);

    let filepath = args.filepath.display().to_string();

    if let Err(e) = _expand(args, source.source()) {
        for e in e {
            let render = e.render(&source, &filepath);

            println!("{}", render);
        }

        return None;
    };

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
        Commands::Expand(assemble_args) => {
            let mut asm = String::new();
            File::open(&assemble_args.filepath)
                .unwrap()
                .read_to_string(&mut asm)?;

            let asm = asm.replace("\t", "    ");

            expand(assemble_args, &asm);
        }
    }

    Ok(())
}
