use std::collections::HashMap;

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

        if let Some(label) = inst.label {
            span = label.span.union(span);
        }

        if let Some(operand) = inst.operand.as_ref() {
            span = operand.0.union(span);
        }

        map.insert(i, span);
    }

    map
}

pub fn assemble_lmc_asm<'a>(asm: &'a str) -> CompilerResult<(Vec<Inst>, SourceMap)> {
    let tokens = tokenize_lmc_asm(asm)?;

    let tokens = expand_lmc_macros(tokens);

    let tokens = {
        let mut new_tokens = vec![];
        let mut line_tokens = vec![];
        for token in tokens.into_iter() {
            if let TokenType::LineEnd = token.type_ {
                new_tokens.push(line_tokens);
                line_tokens = vec![];
                continue;
            }

            line_tokens.push(token);
        }

        new_tokens
            .into_iter()
            .enumerate()
            .filter_map(|(i, line)| {
                if line.len() > 0 {
                    Some((i, line))
                } else {
                    None
                }
            })
            .collect()
    };

    let (raw_insts, symbol_table) = parse_lmc_asm(tokens)?;

    let source_map = generate_source_map(&raw_insts);

    process_labels(raw_insts, symbol_table).map(move |insts| (insts, source_map))
}
