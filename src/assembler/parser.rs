use std::collections::HashMap;

use crate::{
    error::{CompilerError, CompilerResult},
    instruction::{OpCode, Operand, RawInst},
    span::Span,
};

use super::tokenizer::Token;

pub struct Symbol<'a> {
    pub token: Token<'a>,
    pub addr: usize,
}

pub type SymbolTable<'a> = HashMap<&'a str, Symbol<'a>>;

pub fn parse_lmc_asm<'a>(
    tokens: Vec<Vec<Token<'a>>>,
) -> CompilerResult<(Vec<RawInst<'a>>, SymbolTable)> {
    let mut symbol_table = SymbolTable::new();

    let mut insts = vec![];
    let mut errors = vec![];

    'lines: for (i, line) in tokens.into_iter().enumerate() {
        let mut opcode = None;
        let mut operand = None;
        let mut label = None;

        let mut token_iter = line.iter();

        'tokens: while let Some(token) = token_iter.next() {
            if operand.is_some() {
                errors.push(CompilerError::UnexpectedTokens(Span::new(
                    token.span.start,
                    token_iter.last().unwrap_or(token).span.end,
                    token.span.line,
                )));

                continue 'lines;
            }

            if let Ok(op) = token.source.parse::<OpCode>() {
                if opcode.is_some() {
                    errors.push(CompilerError::InvalidLabel(token.source, token.span));

                    continue 'lines;
                }
                opcode = Some((token.span, op));

                continue 'tokens;
            }

            if opcode.is_none() {
                label = Some(*token);

                continue 'tokens;
            }

            if opcode.is_some() {
                operand = Some((
                    token.span,
                    match Operand::lifetime_from_str(token.source)
                        .map_err(|e| CompilerError::OperandParseError(token.span, e))
                    {
                        Ok(operand) => operand,
                        Err(e) => {
                            errors.push(e);

                            continue 'lines;
                        }
                    },
                ))
            }
        }

        if opcode.is_none() {
            let mut span = line.last().unwrap().span;

            let position = span.end;

            span.start = position + 1;
            span.end = position + 1 + 4;

            errors.push(CompilerError::ExpectedOpCode(span));

            continue 'lines;
        }

        if let Some(label) = label {
            if let Some(other) = symbol_table.get(label.source) {
                errors.push(CompilerError::DuplicateLabel(
                    label.source,
                    other.token.span,
                    label.span,
                ));

                continue 'lines;
            }

            symbol_table.insert(
                label.source,
                Symbol {
                    token: label,
                    addr: i,
                },
            );
        }

        insts.push(RawInst {
            label,
            opcode: opcode.unwrap(),
            operand,
        })
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok((insts, symbol_table))
    }
}
