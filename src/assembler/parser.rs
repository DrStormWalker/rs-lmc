use std::{collections::HashMap, str::FromStr};

use crate::{
    error::{CompilerError, CompilerResult},
    instruction::{OpCode, Operand, OperandParseError, OperandValue, RawInst},
    span::Span,
};

use super::tokenizer::{Token, TokenType};

#[derive(Debug)]
pub struct Symbol<'a> {
    pub token: Token<'a>,
    pub addr: usize,
}

pub type SymbolTable<'a> = HashMap<String, Symbol<'a>>;

pub fn parse_lmc_instruction<'a>(
    tokens: Vec<Token<'a>>,
    line: usize,
) -> CompilerResult<RawInst<'a>> {
    let mut token_iter = tokens.iter();

    let token = token_iter
        .next()
        .ok_or(vec![CompilerError::ExpectedOpCode(Span::new(0, 3, line))])?;

    if token.type_ != TokenType::Ident {
        return Err(vec![CompilerError::InvalidToken {
            token: token.source.to_string(),
            span: token.span,
            expected: vec!["label", "opcode"],
        }]);
    }

    let (opcode, label) = if let Ok(op) = token.source.parse::<OpCode>() {
        ((token.span, op), None)
    } else {
        let label = Some(token);

        let token = token_iter
            .next()
            .ok_or(vec![CompilerError::ExpectedOpCode(Span::new(
                token.span.end + 1,
                token.span.end + 4,
                token.span.line,
            ))])?;

        let Ok(op) = token.source.parse::<OpCode>() else {
            return Err(vec![CompilerError::ExpectedOpCode(token.span)]);
        };

        ((token.span, op), label)
    };

    let Some(token) = token_iter.next() else {
        return Ok(RawInst {
            label: label.cloned(),
            opcode,
            operand: None,
        });
    };

    if let Ok(_) = token.source.parse::<OpCode>() {
        return Err(vec![CompilerError::InvalidLabel(
            token.source.to_string(),
            token.span,
        )]);
    }

    match token.type_ {
        TokenType::At | TokenType::Hash => {
            let prefix = token;

            let Some(token) = token_iter.next() else {
                return Err(vec![CompilerError::ExpectedToken(
                    Span::new(prefix.span.end, prefix.span.end + 5, line),
                    vec!["literal", "label"],
                )]);
            };

            match token.type_ {
                TokenType::Literal | TokenType::Ident => {}
                _ => {
                    return Err(vec![CompilerError::InvalidToken {
                        token: token.source.to_string(),
                        span: token.span,
                        expected: vec!["literal", "label"],
                    }])
                }
            }

            let operand = OperandValue::lifetime_from_str(token.source.clone())
                .map_err(|e| vec![CompilerError::OperandParseError(token.span, e)])?;

            let operand = match prefix.type_ {
                TokenType::At => Operand::Indirect(operand),
                TokenType::Hash => Operand::Immediate(operand),
                _ => unreachable!(),
            };

            if !token_iter.is_empty() {
                let next = token_iter.next().unwrap();

                let last = token_iter.last().unwrap_or(&next);

                return Err(vec![CompilerError::UnexpectedTokens(
                    next.span.union(last.span),
                )]);
            }

            Ok(RawInst {
                label: label.cloned(),
                opcode,
                operand: Some((token.span, operand)),
            })
        }
        TokenType::Literal | TokenType::Ident => {
            if !token_iter.is_empty() {
                let next = token_iter.next().unwrap();

                let last = token_iter.last().unwrap_or(&next);

                return Err(vec![CompilerError::UnexpectedTokens(
                    next.span.union(last.span),
                )]);
            }

            Ok(RawInst {
                label: label.cloned(),
                opcode,
                operand: Some((
                    token.span,
                    Operand::lifetime_from_str(token.source.clone())
                        .map_err(|e| vec![CompilerError::OperandParseError(token.span, e)])?,
                )),
            })
        }
        _ => Err(vec![CompilerError::InvalidToken {
            token: token.source.to_string(),
            span: token.span,
            expected: vec!["'@'", "'#'", "literal", "label"],
        }]),
    }
}

pub fn parse_lmc_asm<'a>(
    tokens: Vec<(usize, Vec<Token<'a>>)>,
) -> CompilerResult<(Vec<RawInst<'a>>, SymbolTable)> {
    let mut symbol_table = SymbolTable::new();

    let mut insts = vec![];
    let mut errors = vec![];

    for (i, line) in tokens.into_iter() {
        let inst = match parse_lmc_instruction(line, i) {
            Ok(inst) => inst,
            Err(mut e) => {
                errors.append(&mut e);
                continue;
            }
        };

        if let Some(label) = inst.label.clone() {
            if let Some(other) = symbol_table.get(&label.source[..]) {
                errors.push(CompilerError::DuplicateLabel(
                    label.source.to_string(),
                    other.token.span,
                    label.span,
                ));

                continue;
            }

            symbol_table.insert(
                label.source.to_string(),
                Symbol {
                    token: label,
                    addr: i,
                },
            );
        };

        insts.push(inst);
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok((insts, symbol_table))
    }
}
