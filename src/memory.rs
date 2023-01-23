// use std::{cell::RefCell, iter};

// use crate::instruction::{Inst, OpCode, Operand};

use std::{cell::RefCell, iter};

use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

use crate::instruction::{Inst, OpCode, Value};

// #[derive(Debug, Copy, Clone)]
// pub enum MemoryValue {
//     Instruction(Inst),
//     Value(Value),
// }

// pub trait MemoryStorage {
//     fn get(&self, addr: usize) -> Option<&MemoryValue>;
//     fn get_mut(&mut self, addr: usize) -> Option<&mut MemoryValue>;

//     fn try_extend(&mut self, addr: usize) -> Option<()>;
// }

// #[derive(Debug)]
// pub struct SizedMemoryStorage {
//     memory: Box<[MemoryValue]>,
// }
// impl MemoryStorage for SizedMemoryStorage {
//     fn get(&self, addr: usize) -> Option<&MemoryValue> {
//         self.memory.get(addr)
//     }

//     fn get_mut(&mut self, addr: usize) -> Option<&mut MemoryValue> {
//         self.memory.get_mut(addr)
//     }

//     fn try_extend(&mut self, addr: usize) -> Option<()> {
//         if addr < self.memory.len() {
//             Some(())
//         } else {
//             None
//         }
//     }
// }

// #[derive(Debug)]
// pub struct DynamicMemoryStorage {
//     memory: Vec<MemoryValue>,
// }
// impl MemoryStorage for DynamicMemoryStorage {
//     fn get(&self, addr: usize) -> Option<&MemoryValue> {
//         self.memory.get(addr)
//     }

//     fn get_mut(&mut self, addr: usize) -> Option<&mut MemoryValue> {
//         self.memory.get_mut(addr)
//     }

//     fn try_extend(&mut self, addr: usize) -> Option<()> {
//         if addr < self.memory.len() {
//             return Some(());
//         }

//         let additional = addr - self.memory.len() + 1;

//         self.memory
//             .extend(iter::repeat(MemoryValue::Value(Value::Number(0))).take(additional));

//         Some(())
//     }
// }

// pub type SizedMemory = Memory<SizedMemoryStorage>;
// impl SizedMemory {
//     pub fn with_size(size: usize) -> Self {
//         Self(SizedMemoryStorage {
//             memory: vec![MemoryValue::Value(Value::Number(0)); size].into_boxed_slice(),
//         })
//     }
// }

// pub type DynamicMemory = Memory<DynamicMemoryStorage>;
// impl DynamicMemory {
//     pub fn new() -> Self {
//         Self(DynamicMemoryStorage { memory: vec![] })
//     }
// }

// #[derive(Debug)]
// pub struct Memory<M>(M);
// impl<M: MemoryStorage> Memory<M> {
//     pub fn get(&mut self, addr: usize) -> Option<MemoryValue> {
//         self.0.try_extend(addr)?;
//         self.0.get(addr).copied()
//     }

//     pub fn set(&mut self, addr: usize, value: MemoryValue) -> Option<MemoryValue> {
//         self.0.try_extend(addr)?;
//         let mem = self.0.get_mut(addr)?;

//         let previous = *mem;

//         *mem = value;

//         Some(previous)
//     }

//     pub fn get_value(&mut self, addr: usize) -> Option<Value> {
//         let value = self.get(addr)?;

//         match value {
//             MemoryValue::Value(v) => Some(v),
//             MemoryValue::Instruction(inst) => Some(inst.into()),
//         }
//     }

//     pub fn get_inst(&mut self, addr: usize) -> Option<Inst> {
//         let value = self.get(addr)?;

//         match value {
//             MemoryValue::Instruction(inst) => Some(inst),
//             MemoryValue::Value(v) => Some(v.into()),
//         }
//     }

//     pub fn load_program(&mut self, insts: Vec<Inst>) -> Option<()> {
//         for (i, inst) in insts.into_iter().enumerate() {
//             let mem_value = match inst {
//                 Inst {
//                     opcode: OpCode::Dat,
//                     operand,
//                 } => MemoryValue::Value(match operand {
//                     None => Value::Number(0),
//                     Some(Operand::Addr(addr)) => Value::Number(addr),
//                     _ => Value::NaN,
//                 }),
//                 _ => MemoryValue::Instruction(inst),
//             };

//             self.set(i, mem_value)?;
//         }

//         Some(())
//     }
// }

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Address `{0}` is outside the range of valid addresses, `0-{1}`")]
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
