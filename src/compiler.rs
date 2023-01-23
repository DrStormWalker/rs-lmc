use std::collections::HashMap;

use crate::{
    error::{CompilerError, CompilerResult},
    instruction::{Inst, OpCode, Operand, OperandValue, RawInst},
    span::Span,
};

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    source: &'a str,
    span: Span,
}

fn tokenize_lmc_asm<'a>(asm: &'a str) -> Vec<Vec<Token>> {
    let lines = asm.lines();

    lines
        .enumerate()
        .filter_map(|(i, line)| {
            let mut line_tokens = vec![];

            let mut token_start = 0;

            let mut last_whitespace = false;

            for (j, char) in line.char_indices() {
                if last_whitespace {
                    token_start = j;
                }

                last_whitespace = false;

                if char.is_whitespace() {
                    if token_start != j {
                        line_tokens.push(Token {
                            source: &line[token_start..j],
                            span: Span::new(token_start, j, i),
                        });
                    }

                    last_whitespace = true;
                }
            }

            let last_token = &line[token_start..line.len()].trim();

            if !last_token.is_empty() && !last_whitespace {
                line_tokens.push(Token {
                    source: &last_token,
                    span: Span::new(token_start, line.len(), i),
                });
            }

            if line_tokens.len() > 0 {
                Some(line_tokens)
            } else {
                None
            }
        })
        .collect()
}

struct Symbol<'a> {
    token: Token<'a>,
    addr: usize,
}

type SymbolTable<'a> = HashMap<&'a str, Symbol<'a>>;

fn parse_lmc_asm<'a>(tokens: Vec<Vec<Token<'a>>>) -> CompilerResult<(Vec<RawInst>, SymbolTable)> {
    let mut symbol_table = SymbolTable::new();

    let insts = tokens
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            if line.len() > 3 {
                let first_token = line.get(3).unwrap();
                let last_token = line.last().unwrap();

                let span = first_token.span.union(last_token.span);

                return Err(CompilerError::UnexpectedTokens(span));
            }

            if line.len() < 1 {
                unreachable!("Empty line");
            }

            let mut label = None;
            let mut opcode = None;
            let mut operand = None;

            for token in line.iter() {
                if let Ok(op) = token.source.parse::<OpCode>() {
                    if opcode.is_some() {
                        return Err(CompilerError::InvalidLabel(token.source, token.span));
                    }

                    opcode = Some((token.span, op));

                    continue;
                }

                if opcode.is_none() {
                    label = Some(*token);

                    continue;
                }

                if opcode.is_some() {
                    operand = Some((
                        token.span,
                        Operand::lifetime_from_str(token.source)
                            .map_err(|e| CompilerError::OperandParseError(token.span, e))?,
                    ));

                    continue;
                }
            }

            let opcode = opcode.ok_or_else(move || {
                let mut span = line.last().unwrap().span;

                let mut position = span.end;

                position += 1;

                span.start = position;

                position += 4;

                span.end = position;

                CompilerError::ExpectedOpCode(span)
            })?;

            if let Some(label) = label {
                if let Some(other) = symbol_table.get(label.source) {
                    return Err(CompilerError::DuplicateLabel(
                        label.source,
                        other.token.span,
                        label.span,
                    ));
                }

                symbol_table.insert(
                    label.source,
                    Symbol {
                        token: label,
                        addr: i,
                    },
                );
            }

            Ok(RawInst {
                label,
                opcode,
                operand,
            })
        })
        .collect::<CompilerResult<Vec<RawInst>>>()?;

    Ok((insts, symbol_table))
}

fn process_labels<'a>(
    insts: Vec<RawInst<'a>>,
    symbol_table: SymbolTable,
) -> CompilerResult<'a, Vec<Inst>> {
    insts
        .into_iter()
        .map(|inst| {
            let operand = inst
                .operand
                .map(|(span, operand)| {
                    operand.try_map_value(|v| match v {
                        OperandValue::Value(v) => Ok(v),
                        OperandValue::Label(label) => symbol_table
                            .get(&label[..])
                            .map(|symbol| symbol.addr as i64)
                            .ok_or_else(|| CompilerError::UndefinedLabel(&label, span)),
                    })
                })
                .transpose()?;

            Ok(Inst {
                opcode: inst.opcode.1,
                operand,
            })
        })
        .collect()
}

pub fn compile_lmc_asm<'a>(asm: &'a str) -> CompilerResult<Vec<Inst>> {
    let tokens = tokenize_lmc_asm(asm);

    let (raw_insts, symbol_table) = parse_lmc_asm(tokens)?;

    process_labels(raw_insts, symbol_table)
}
