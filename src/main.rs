mod error;
mod instruction;
mod interpreter;
mod span;

use std::{fs::File, io::Read, path::PathBuf};

use clap::{Parser, Subcommand};
use error::InterpreterError;
use instruction::OperandParseError;
use interpreter::compile_lmc_asm;
use span::SourceBuffer;

use crate::span::RenderLabel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run { filepath: PathBuf },
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Run { filepath } => {
            let mut asm = String::new();
            File::open(filepath).unwrap().read_to_string(&mut asm);

            let source = SourceBuffer::new(&asm);

            if let Err(e) = compile_lmc_asm(&source.source()) {
                // match e {
                //     InterpreterError::UnexpectedTokens(span) => {
                //         println!(
                //             "\n{}",
                //             span.render(
                //                 &source,
                //                 Some(RenderLabel::Error("Unexpected tokens")),
                //                 &filepath.display().to_string(),
                //                 &[],
                //             ),
                //         );
                //     }
                //     InterpreterError::InvalidLabel(label, span) => {
                //         println!(
                //             "\n{}",
                //             span.render(
                //                 &source,
                //                 Some(RenderLabel::Error(&format!("Invalid label `{}`", label))),
                //                 &filepath.display().to_string(),
                //                 &[&format!(
                //                     "Keywords such as `{}` are not allowed to be used as labels",
                //                     label,
                //                 )],
                //             ),
                //         );
                //     }
                //     InterpreterError::ExpectedOpCode(span) => {
                //         println!(
                //             "\n{}",
                //             span.render(
                //                 &source,
                //                 Some(RenderLabel::Error("Expected an opcode")),
                //                 &filepath.display().to_string(),
                //                 &["All parts of an instruction must be on the same line"],
                //             ),
                //         );
                //     }
                //     InterpreterError::OperandParseError(span, e) => match e {
                //         OperandParseError::InvalidIntegerLiteral(e) => {
                //             println!(
                //                 "\n{}",
                //                 span.render(
                //                     &source,
                //                     Some(RenderLabel::Error("Invalid integer literl")),
                //                     &filepath.display().to_string(),
                //                     &[&e.to_string()],
                //                 ),
                //             );
                //         }
                //         OperandParseError::InvalidLabel(label) => {
                //             println!(
                //                 "\n{}",
                //                 span.render(
                //                     &source,
                //                     Some(RenderLabel::Error("Invalid label")),
                //                     &filepath.display().to_string(),
                //                     &[&format!(
                //                         "The label `{}` contains invalid characters",
                //                         label
                //                     )],
                //                 ),
                //             );
                //         }
                //     },
                // }

                println!();

                println!("{}", e.render(&source, &filepath.display().to_string()));
            }
        }
    }
}
