use thiserror::Error;

use crate::{
    instruction::{OpCode, Operand, OperandParseError, RawInst},
    span::{Position, Span},
};

#[derive(Clone, Debug, Error)]
pub enum InterpreterError<'a> {
    #[error("Unexpected tokens")]
    UnexpectedTokens(Span),

    #[error("Invalid label name")]
    InvalidLabel(&'a str, Span),

    #[error("Expected opcode")]
    ExpectedOpCode(Span),

    #[error("{1}")]
    OperandParseError(Span, OperandParseError),
}

pub type InterpreterResult<'a, T> = Result<T, InterpreterError<'a>>;

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    source: &'a str,
    span: Span,
}

pub fn tokenize_lmc_asm<'a>(asm: &'a str) -> InterpreterResult<Vec<Vec<Token>>> {
    let lines = asm.lines();

    let mut tokens = vec![];

    for (i, line) in lines.enumerate() {
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
                        span: Span::new(Position::new(i, token_start), Position::new(i, j)),
                    });
                }

                last_whitespace = true;
            }
        }

        let last_token = &line[token_start..line.len()].trim();

        if !last_token.is_empty() && !last_whitespace {
            line_tokens.push(Token {
                source: &last_token,
                span: Span::new(Position::new(i, token_start), Position::new(i, line.len())),
            });
        }

        if line_tokens.len() > 0 {
            tokens.push(line_tokens);
        }
    }

    Ok(tokens)
}

pub fn parse_lmc_asm<'a>(tokens: Vec<Vec<Token<'a>>>) -> InterpreterResult<()> {
    for line in tokens {
        if line.len() > 3 {
            let first_token = line.get(3).unwrap();
            let last_token = line.last().unwrap();

            let span = first_token.span.union(last_token.span);

            return Err(InterpreterError::UnexpectedTokens(span));
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
                    return Err(InterpreterError::InvalidLabel(token.source, token.span));
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
                    token
                        .source
                        .parse::<Operand>()
                        .map_err(|e| InterpreterError::OperandParseError(token.span, e))?,
                ));

                continue;
            }
        }

        let opcode = opcode.ok_or_else(move || {
            let mut span = line.last().unwrap().span;

            let mut position = span.end;

            position.char += 1;

            span.start = position;

            position.char += 4;

            span.end = position;

            InterpreterError::ExpectedOpCode(span)
        })?;

        let raw_inst = RawInst {
            label,
            opcode,
            operand,
        };

        println!("{:?}", raw_inst);
    }

    Ok(())
}

pub fn compile_lmc_asm<'a>(asm: &'a str) -> InterpreterResult<()> {
    let tokens = tokenize_lmc_asm(asm)?;

    parse_lmc_asm(tokens)?;

    Ok(())
}
