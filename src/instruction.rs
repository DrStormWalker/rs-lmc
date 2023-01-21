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
pub(crate) enum OperandValue<'a> {
    Label(&'a str),
    Value(i64),
}
impl<'a> OperandValue<'a> {
    pub fn from_str(s: &'a str) -> Result<Self, OperandParseError> {
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
            Ok(Self::Label(s))
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
pub(crate) enum Operand<T> {
    Addr(T),
    Immediate(T),
    Indirect(T),
}
impl<T> Operand<T> {
    pub fn try_map_value<B, E, F>(self, f: F) -> Result<Operand<B>, E>
    where
        F: FnOnce(T) -> Result<B, E>,
    {
        Ok(match self {
            Self::Addr(v) => Operand::Addr(f(v)?),
            Self::Immediate(v) => Operand::Immediate(f(v)?),
            Self::Indirect(v) => Operand::Indirect(f(v)?),
        })
    }
}
impl<'a> Operand<OperandValue<'a>> {
    pub fn from_str(s: &'a str) -> Result<Self, OperandParseError> {
        if s.starts_with("#") {
            Ok(Operand::Immediate(OperandValue::from_str(&s[1..])?))
        } else if s.starts_with("@") {
            Ok(Operand::Indirect(OperandValue::from_str(&s[1..])?))
        } else {
            Ok(Operand::Addr(OperandValue::from_str(&s[..])?))
        }
    }
}

#[derive(Debug)]
pub(crate) struct RawInst<'a> {
    pub(crate) label: Option<Token<'a>>,
    pub(crate) opcode: (Span, OpCode),
    pub(crate) operand: Option<(Span, Operand<OperandValue<'a>>)>,
}

#[derive(Debug)]
pub struct Inst {
    pub(crate) opcode: OpCode,
    pub(crate) operand: Option<Operand<i64>>,
}
