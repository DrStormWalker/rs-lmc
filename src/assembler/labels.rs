use crate::{
    error::{CompilerError, CompilerResult},
    instruction::{Inst, OperandValue, RawInst},
};

use super::parser::SymbolTable;

pub fn process_labels<'a>(
    insts: Vec<RawInst<'a>>,
    symbol_table: SymbolTable,
) -> CompilerResult<'a, Vec<Inst>> {
    let mut new_insts = vec![];
    let mut errors = vec![];

    for inst in insts.into_iter() {
        let operand = match inst
            .operand
            .map(|(span, operand)| {
                operand.try_map_value(|v| match v {
                    OperandValue::Value(v) => Ok(v),
                    OperandValue::Label(label) => symbol_table
                        .get(&label[..])
                        .map(|symbol| symbol.addr as i64)
                        .ok_or_else(|| CompilerError::UndefinedLabel(label.to_string(), span)),
                })
            })
            .transpose()
        {
            Ok(operand) => operand,
            Err(e) => {
                errors.push(e);

                continue;
            }
        };

        new_insts.push(Inst {
            opcode: inst.opcode.1,
            operand,
        })
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(new_insts)
    }
}
