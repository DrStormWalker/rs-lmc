use std::collections::HashMap;

use regex::Regex;

use crate::{
    error::CompilerResult,
    instruction::{Inst, RawInst},
    span::Span,
};

use self::{
    labels::process_labels,
    macros::expand_lmc_macros,
    parser::parse_lmc_asm,
    tokenizer::{tokenize_lmc_asm, TokenType},
};

pub mod labels;
pub mod macros;
pub mod parser;
pub mod tokenizer;

pub type SourceMap = HashMap<usize, Span>;

fn generate_source_map<'a>(insts: &[RawInst<'a>]) -> SourceMap {
    let mut map = SourceMap::new();

    for (i, inst) in insts.iter().enumerate() {
        let mut span = inst.opcode.0;

        if let Some(label) = &inst.label {
            span = label.span.union(span);
        }

        if let Some(operand) = inst.operand.as_ref() {
            span = operand.0.union(span);
        }

        map.insert(i, span);
    }

    map
}

pub fn assemble_lmc_asm<'a>(asm: &'a str, print: bool) -> CompilerResult<(Vec<Inst>, SourceMap)> {
    let tokens = tokenize_lmc_asm(asm)?;

    let tokens = expand_lmc_macros(tokens)?;

    lazy_static::lazy_static! {
        static ref REMOVE_SPACE_RE: Regex = Regex::new(r"(?P<tok>[\n#@]) ").unwrap();
        static ref COLLAPSE_LINE_RE: Regex = Regex::new(r"(\n\s*){2,}").unwrap();
        static ref TRAILING_WHITESPACE: Regex = Regex::new(r"\s*$").unwrap();
        static ref PRECEDING_WHITESPACE: Regex = Regex::new(r"^\s*").unwrap();
    }

    if print {
        println!(
            "=== EXPANDED PROGRAM ===\n\n{}\n\n=== END EXPANDED PROGRAM ===\n",
            REMOVE_SPACE_RE.replace_all(
                &COLLAPSE_LINE_RE.replace_all(
                    &TRAILING_WHITESPACE.replace(
                        &PRECEDING_WHITESPACE.replace(
                            &tokens
                                .iter()
                                .map(|t| &t.source[..])
                                .collect::<Vec<&str>>()
                                .join(" "),
                            "",
                        ),
                        "",
                    ),
                    "\n\n",
                ),
                "$tok"
            )
        );
    }

    let tokens = {
        let mut new_tokens = vec![];
        let mut line_tokens = vec![];
        for token in tokens.into_iter() {
            if token.type_ == TokenType::LineEnd {
                new_tokens.push(line_tokens);
                line_tokens = vec![];
                continue;
            }

            line_tokens.push(token);
        }

        new_tokens.push(line_tokens);

        new_tokens
            .into_iter()
            .filter(|line| line.len() > 0)
            .enumerate()
            .collect()
    };

    let (raw_insts, symbol_table) = parse_lmc_asm(tokens)?;

    let source_map = generate_source_map(&raw_insts);

    process_labels(raw_insts, symbol_table).map(move |insts| (insts, source_map))
}
