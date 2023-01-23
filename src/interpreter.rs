use std::{
    fmt::Debug,
    io::{self, Error as IoError, Write},
};

use thiserror::Error;

use crate::{
    instruction::{Inst, OpCode, Operand, Value},
    memory::{MemoryError, MemoryStorage},
};

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error(transparent)]
    MemoryError(#[from] MemoryError),

    #[error("Attempted to use a NaN value as an address")]
    NaNAddrError,

    #[error("Attempted to use an Inf value as an address")]
    InfAddrError,

    #[error("Attempted to use a -Inf value as an address")]
    NegInfAddrError,

    #[error("Attemped to use a negative value as an adress")]
    NegAddrError,

    #[error(transparent)]
    IoError(#[from] IoError),
}

pub type InterpreterResult<T> = Result<T, InterpreterError>;

pub struct Vm<M> {
    pc: usize,
    mar: usize,
    mdr: Value,
    cir: Inst,
    acc: Value,

    memory: M,
}
impl<M: MemoryStorage + Debug> Vm<M> {
    pub fn new(memory: M) -> Self {
        Self {
            pc: 0,
            mar: 0,
            mdr: Value::Number(0),
            cir: Inst {
                opcode: OpCode::Nop,
                operand: None,
            },
            acc: Value::Number(0),

            memory,
        }
    }

    pub fn run_program(&mut self) -> InterpreterResult<()> {
        loop {
            self.fetch_decode()?;

            // println!(
            //     "pc: {:?}, mar: {:?}, mdr: {:?}, cir: {:?}, acc: {:?}",
            //     self.pc, self.mar, self.mdr, self.cir, self.acc
            // );

            // println!("{:?}", self.memory);

            if self.cir.opcode == OpCode::Hlt {
                break Ok(());
            }

            self.execute()?;
        }
    }

    fn get_operand_value(&mut self, operand: Operand<i64>) -> InterpreterResult<Value> {
        match operand {
            Operand::Addr(addr) => Ok(self.memory.get_value(
                addr.try_into()
                    .map_err(|_| InterpreterError::NegAddrError)?,
            )?),
            Operand::Immediate(imm) => Ok(Value::Number(imm)),
            Operand::Indirect(addr) => {
                let addr = self.memory.get_value(
                    addr.try_into()
                        .map_err(|_| InterpreterError::NegAddrError)?,
                )?;

                let addr = match addr {
                    Value::Number(addr) => addr,
                    Value::InfPositive => return Err(InterpreterError::InfAddrError),
                    Value::InfNegative => return Err(InterpreterError::NegInfAddrError),
                    Value::NaN => return Err(InterpreterError::NaNAddrError),
                };

                Ok(self.memory.get_value(
                    addr.try_into()
                        .map_err(|_| InterpreterError::NegAddrError)?,
                )?)
            }
        }
    }

    fn get_operand_addr(&mut self, operand: Operand<i64>) -> InterpreterResult<Value> {
        match operand {
            Operand::Addr(addr) => Ok(Value::Number(addr)),
            Operand::Immediate(imm) => Ok(Value::Number(imm)),
            Operand::Indirect(addr) => {
                let addr = self.memory.get_value(
                    addr.try_into()
                        .map_err(|_| InterpreterError::NegAddrError)?,
                )?;

                Ok(addr)
            }
        }
    }

    pub fn fetch_decode(&mut self) -> InterpreterResult<()> {
        self.cir = self.memory.get_inst(self.pc)?;
        self.pc += 1;

        Ok(())
    }

    pub fn break_to(&mut self, addr: Value) -> InterpreterResult<()> {
        match addr {
            Value::Number(addr) => {
                self.pc = addr
                    .try_into()
                    .map_err(|_| InterpreterError::NegAddrError)?;
            }
            Value::InfPositive => return Err(InterpreterError::InfAddrError),
            Value::InfNegative => return Err(InterpreterError::NegInfAddrError),
            Value::NaN => return Err(InterpreterError::NaNAddrError),
        }

        Ok(())
    }

    pub fn execute(&mut self) -> InterpreterResult<()> {
        match self.cir.opcode {
            OpCode::Inp => {
                let mut buffer = String::new();

                print!("INPUT > ");
                io::stdout().flush()?;
                io::stdin().read_line(&mut buffer)?;
                self.acc = buffer.parse().unwrap();
            }
            OpCode::Out => {
                println!("{}", self.acc.to_string());
            }
            OpCode::Add => {
                if let Some(operand) = self.cir.operand {
                    let value = self.get_operand_value(operand)?;

                    self.acc += value;
                }
            }
            OpCode::Sub => {
                if let Some(operand) = self.cir.operand {
                    let value = self.get_operand_value(operand)?;

                    self.acc -= value;
                }
            }
            OpCode::Sta => {
                if let Some(operand) = self.cir.operand {
                    let value = self.get_operand_addr(operand)?;

                    match value {
                        Value::Number(addr) => self.memory.set_value(
                            addr.try_into()
                                .map_err(|_| InterpreterError::NegAddrError)?,
                            self.acc,
                        )?,
                        Value::InfPositive => return Err(InterpreterError::InfAddrError),
                        Value::InfNegative => return Err(InterpreterError::NegInfAddrError),
                        Value::NaN => return Err(InterpreterError::NaNAddrError),
                    }
                }
            }
            OpCode::Lda => {
                if let Some(operand) = self.cir.operand {
                    let value = self.get_operand_value(operand)?;

                    self.acc = value;
                }
            }
            OpCode::Bra => {
                if let Some(operand) = self.cir.operand {
                    let value = self.get_operand_addr(operand)?;

                    self.break_to(value)?;
                }
            }
            OpCode::Brz => match self.acc {
                Value::Number(0) => {
                    if let Some(operand) = self.cir.operand {
                        let value = self.get_operand_addr(operand)?;

                        self.break_to(value)?;
                    }
                }
                _ => {}
            },
            OpCode::Brp => match self.acc {
                Value::Number(value) => {
                    if value >= 0 {
                        if let Some(operand) = self.cir.operand {
                            let value = self.get_operand_addr(operand)?;

                            self.break_to(value)?;
                        }
                    }
                }
                Value::InfPositive => {
                    if let Some(operand) = self.cir.operand {
                        let value = self.get_operand_value(operand)?;

                        self.break_to(value)?;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
}
