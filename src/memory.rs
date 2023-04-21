// use std::{cell::RefCell, iter};

// use crate::instruction::{Inst, OpCode, Operand};

use std::{cell::RefCell, iter};

use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use crate::instruction::{Inst, OpCode, Value};

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error(
        "attempted to access address `{0}` which outside the range of valid addresses, `0-{1}`"
    )]
    AddrOutOfRange(usize, usize),
}

pub type MemoryResult<T> = Result<T, MemoryError>;

pub trait MemoryStorage {
    fn get_value(&self, addr: usize) -> MemoryResult<Value>;
    fn get_inst(&self, addr: usize) -> MemoryResult<Inst>;

    fn set_value(&mut self, addr: usize, value: Value) -> MemoryResult<()>;
    fn set_inst(&mut self, addr: usize, inst: Inst) -> MemoryResult<()>;
}

pub trait StorageCell {
    fn empty() -> Self;

    fn get_value(&self) -> Value;
    fn get_inst(&self) -> Inst;

    fn set_value(&mut self, value: Value);
    fn set_inst(&mut self, inst: Inst);
}

lazy_static! {
    static ref VALUE_RE: Regex = Regex::new("^-?(Inf|NaN|[0-9]+)").unwrap();
}

impl StorageCell for String {
    fn get_value(&self) -> Value {
        if self.is_empty() {
            return Value::Number(0);
        }

        let value = VALUE_RE.find(self).unwrap();

        let value = &self[value.start()..value.end()];

        match value {
            "Inf" => Value::InfPositive,
            "-Inf" => Value::InfNegative,
            "NaN" => Value::NaN,
            _ => Value::Number(self.parse().unwrap()),
        }
    }

    fn get_inst(&self) -> Inst {
        if self.is_empty() {
            return Inst {
                opcode: OpCode::Hlt,
                operand: None,
            };
        }

        match &self[..1] {
            "0" => Inst {
                opcode: OpCode::Hlt,
                operand: None,
            },
            "1" => Inst {
                opcode: OpCode::Add,
                operand: Some(self[1..].parse().unwrap()),
            },
            "2" => Inst {
                opcode: OpCode::Sub,
                operand: Some(self[1..].parse().unwrap()),
            },
            "3" => Inst {
                opcode: OpCode::Sta,
                operand: Some(self[1..].parse().unwrap()),
            },
            "5" => Inst {
                opcode: OpCode::Lda,
                operand: Some(self[1..].parse().unwrap()),
            },
            "6" => Inst {
                opcode: OpCode::Bra,
                operand: Some(self[1..].parse().unwrap()),
            },
            "7" => Inst {
                opcode: OpCode::Brz,
                operand: Some(self[1..].parse().unwrap()),
            },
            "8" => Inst {
                opcode: OpCode::Brp,
                operand: Some(self[1..].parse().unwrap()),
            },
            "9" => match &self[1..3] {
                "01" => Inst {
                    opcode: OpCode::Inp,
                    operand: None,
                },
                "02" => Inst {
                    opcode: OpCode::Out,
                    operand: None,
                },
                _ => Inst {
                    opcode: OpCode::Nop,
                    operand: None,
                },
            },
            _ => Inst {
                opcode: OpCode::Nop,
                operand: None,
            },
        }
    }

    fn set_value(&mut self, value: Value) {
        self.clone_from(&value.to_string())
    }

    fn set_inst(&mut self, inst: Inst) {
        self.clone_from(&inst.to_string())
    }

    fn empty() -> Self {
        "0".to_string()
    }
}

#[derive(Debug)]
pub struct DynamicMemoryStorage<C> {
    memory: RefCell<Vec<C>>,
}
impl<C> DynamicMemoryStorage<C> {
    pub fn new() -> Self {
        Self {
            memory: RefCell::new(vec![]),
        }
    }
}
impl<C: StorageCell> DynamicMemoryStorage<C> {
    #[inline]
    fn expand_for(&self, addr: usize) {
        let mut memory = self.memory.borrow_mut();

        if addr >= memory.len() {
            let diff = addr - memory.len() + 1;

            memory.extend(iter::repeat_with(|| C::empty()).take(diff));
        }
    }
}
impl<C: StorageCell> MemoryStorage for DynamicMemoryStorage<C> {
    fn get_value(&self, addr: usize) -> MemoryResult<Value> {
        self.expand_for(addr);

        let memory = self.memory.borrow();

        memory
            .get(addr)
            .map(|c| c.get_value())
            .ok_or(MemoryError::AddrOutOfRange(addr, memory.len() - 1))
    }

    fn get_inst(&self, addr: usize) -> MemoryResult<Inst> {
        self.expand_for(addr);

        let memory = self.memory.borrow();

        memory
            .get(addr)
            .map(|c| c.get_inst())
            .ok_or(MemoryError::AddrOutOfRange(addr, memory.len() - 1))
    }

    fn set_value(&mut self, addr: usize, value: Value) -> MemoryResult<()> {
        self.expand_for(addr);

        let mut memory = self.memory.borrow_mut();

        let len = memory.len();

        let cell = memory
            .get_mut(addr)
            .ok_or(MemoryError::AddrOutOfRange(addr, len - 1))?;

        cell.set_value(value);

        Ok(())
    }

    fn set_inst(&mut self, addr: usize, inst: Inst) -> MemoryResult<()> {
        self.expand_for(addr);

        let mut memory = self.memory.borrow_mut();

        let len = memory.len();

        let cell = memory
            .get_mut(addr)
            .ok_or(MemoryError::AddrOutOfRange(addr, len - 1))?;

        cell.set_inst(inst);

        Ok(())
    }
}
