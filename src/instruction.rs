use std::{num::ParseIntError, str::FromStr};

use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use crate::{interpreter::Token, span::Span};

#[derive(Debug)]
pub enum OpCode {
    Hlt,

    Add,
    Sub,

    Sta,
    Lda,

    Bra,
    Brz,
    Brp,

    Inp,
    Out,

    Dat,
}
impl FromStr for OpCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "hlt" => Ok(Self::Hlt),
            "add" => Ok(Self::Add),
            "sub" => Ok(Self::Sub),
            "sta" => Ok(Self::Sta),
            "lda" => Ok(Self::Lda),
            "bra" => Ok(Self::Bra),
            "brz" => Ok(Self::Brz),
            "brp" => Ok(Self::Brp),
            "inp" => Ok(Self::Inp),
            "out" => Ok(Self::Out),
            "dat" => Ok(Self::Dat),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum OperandValue {
    Label(String),
    Value(i64),
}
impl FromStr for OperandValue {
    type Err = OperandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref LABEL_RE: Regex = Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*$").unwrap();
            static ref NUMBER_START_RE: Regex = Regex::new("^[0-9]").unwrap();
        }

        if !LABEL_RE.is_match(s) {
            if !NUMBER_START_RE.is_match(s) {
                return Err(OperandParseError::InvalidLabel(s.to_string()));
            }

            let value = s.parse::<i64>()?;

            Ok(Self::Value(value))
        } else {
            Ok(Self::Label(s.to_string()))
        }
    }
}

#[derive(Clone, Debug, Error)]
pub enum OperandParseError {
    #[error(transparent)]
    InvalidIntegerLiteral(#[from] ParseIntError),

    #[error("Invalid label {0}")]
    InvalidLabel(String),
}

#[derive(Debug)]
pub(crate) enum Operand {
    Addr(OperandValue),
    Immediate(OperandValue),
    Indirect(OperandValue),
}
impl FromStr for Operand {
    type Err = OperandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("#") {
            Ok(Operand::Immediate(s[1..].parse()?))
        } else if s.starts_with("@") {
            Ok(Operand::Indirect(s[1..].parse()?))
        } else {
            Ok(Operand::Addr(s.parse()?))
        }
    }
}

#[derive(Debug)]
pub(crate) struct RawInst<'a> {
    pub(crate) label: Option<Token<'a>>,
    pub(crate) opcode: (Span, OpCode),
    pub(crate) operand: Option<(Span, Operand)>,
}
