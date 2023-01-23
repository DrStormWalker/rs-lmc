use std::{
    num::ParseIntError,
    ops::{AddAssign, SubAssign},
    str::FromStr,
};

use cached::{proc_macro::cached, SizedCache};
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use crate::{compiler::Token, span::Span};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
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

    Nop,
}
impl OpCode {
    pub fn opcode(&self) -> i64 {
        match self {
            Self::Hlt => 0,
            Self::Add => 1,
            Self::Sub => 2,
            Self::Sta => 3,
            Self::Lda => 5,
            Self::Bra => 6,
            Self::Brz => 7,
            Self::Brp => 8,
            Self::Inp => 901,
            Self::Out => 902,
            Self::Dat => 0,
            Self::Nop => 4,
        }
    }
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
    pub fn lifetime_from_str(s: &'a str) -> Result<Self, OperandParseError> {
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

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
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
    pub fn lifetime_from_str(s: &'a str) -> Result<Self, OperandParseError> {
        if s.starts_with("#") {
            Ok(Operand::Immediate(OperandValue::lifetime_from_str(
                &s[1..],
            )?))
        } else if s.starts_with("@") {
            Ok(Operand::Indirect(OperandValue::lifetime_from_str(&s[1..])?))
        } else {
            Ok(Operand::Addr(OperandValue::lifetime_from_str(&s[..])?))
        }
    }
}
impl<T: ToString> ToString for Operand<T> {
    fn to_string(&self) -> String {
        match self {
            Self::Addr(addr) => addr.to_string(),
            Self::Immediate(addr) => "#".to_string() + &addr.to_string(),
            Self::Indirect(addr) => "@".to_string() + &addr.to_string(),
        }
    }
}
impl<E, T: FromStr<Err = E>> FromStr for Operand<T> {
    type Err = E;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("#") {
            Ok(Operand::Immediate(s[1..].parse()?))
        } else if s.starts_with("@") {
            Ok(Operand::Indirect(s[1..].parse()?))
        } else {
            Ok(Operand::Addr(s[..].parse()?))
        }
    }
}

#[derive(Debug)]
pub(crate) struct RawInst<'a> {
    pub(crate) label: Option<Token<'a>>,
    pub(crate) opcode: (Span, OpCode),
    pub(crate) operand: Option<(Span, Operand<OperandValue<'a>>)>,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Value {
    Number(i64),
    NaN,
    InfPositive,
    InfNegative,
}
impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Self::InfPositive => "Inf".to_string(),
            Self::InfNegative => "-Inf".to_string(),
            Self::NaN => "NaN".to_string(),
            Self::Number(number) => number.to_string(),
        }
    }
}
impl FromStr for Value {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "NaN" => Ok(Self::NaN),
            "Inf" | "+Inf" => Ok(Self::InfPositive),
            "-Inf" => Ok(Self::InfNegative),
            s => Ok(s.parse::<i64>().map_or(Self::NaN, |n| Self::Number(n))),
        }
    }
}
impl AddAssign for Value {
    fn add_assign(&mut self, rhs: Self) {
        *self = match (*self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            (Self::NaN, _) => Self::NaN,
            (Self::InfPositive, _) => Self::InfPositive,
            (Self::InfNegative, _) => Self::InfNegative,
            (_, Self::NaN) => Self::NaN,
            (_, Self::InfPositive) => Self::InfPositive,
            (_, Self::InfNegative) => Self::InfNegative,
        }
    }
}
impl SubAssign for Value {
    fn sub_assign(&mut self, rhs: Self) {
        *self = match (*self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a - b),
            (Self::NaN, _) => Self::NaN,
            (Self::InfPositive, _) => Self::InfPositive,
            (Self::InfNegative, _) => Self::InfNegative,
            (_, Self::NaN) => Self::NaN,
            (_, Self::InfPositive) => Self::InfPositive,
            (_, Self::InfNegative) => Self::InfNegative,
        }
    }
}

#[cached(
    type = "SizedCache<Inst, Value>",
    create = "{ SizedCache::with_size(1000) }"
)]
fn inst_into_value(inst: Inst) -> Value {
    match inst.opcode {
        OpCode::Dat => match inst.operand {
            None => Value::Number(0),
            Some(Operand::Addr(addr)) => Value::Number(addr),
            _ => Value::NaN,
        },
        opcode => match inst.operand {
            Some(Operand::Addr(addr)) => {
                let power = addr.checked_ilog10().unwrap_or(0) + 1;

                let opcode = opcode.opcode() * 10_i64.pow(power);

                Value::Number(opcode + addr)
            }
            _ => Value::Number(opcode.opcode()),
        },
    }
}

#[cached(
    type = "SizedCache<Value, Inst>",
    create = "{ SizedCache::with_size(1000) }"
)]
fn value_into_inst(value: Value) -> Inst {
    match value {
        Value::Number(0) => Inst {
            opcode: OpCode::Hlt,
            operand: None,
        },
        Value::Number(v) => {
            if v < 0 {
                Inst {
                    opcode: OpCode::Nop,
                    operand: None,
                }
            } else {
                let mut log = v.ilog10();
                let mut start = v / log as i64;

                let opcode = match start {
                    1 => OpCode::Add,
                    2 => OpCode::Sub,
                    3 => OpCode::Sta,
                    5 => OpCode::Lda,
                    6 => OpCode::Bra,
                    7 => OpCode::Brz,
                    8 => OpCode::Brp,
                    9 => match log.checked_sub(2) {
                        Some(log_2) => match v / log_2 as i64 {
                            901 => {
                                start = 901;
                                log = log_2;

                                OpCode::Inp
                            }
                            902 => {
                                start = 90264;
                                log = log_2;

                                OpCode::Out
                            }
                            _ => OpCode::Nop,
                        },
                        None => OpCode::Nop,
                    },
                    _ => OpCode::Nop,
                };

                let operand = if log as i64 > start {
                    Some(Operand::Addr(v - start * log as i64))
                } else {
                    None
                };

                Inst { opcode, operand }
            }
        }
        _ => Inst {
            opcode: OpCode::Nop,
            operand: None,
        },
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Inst {
    pub(crate) opcode: OpCode,
    pub(crate) operand: Option<Operand<i64>>,
}
impl Into<Value> for Inst {
    #[inline]
    fn into(self) -> Value {
        inst_into_value(self)
    }
}
impl From<Value> for Inst {
    #[inline]
    fn from(value: Value) -> Self {
        value_into_inst(value)
    }
}
impl ToString for Inst {
    fn to_string(&self) -> String {
        let opcode = self.opcode.opcode().to_string();

        opcode + &self.operand.map_or("".to_string(), |op| op.to_string())
    }
}
