use super::parser::SymbolTable;
use crate::{
    error::CompilerResult,
    instruction::{OpCode, Operand, OperandValue, RawInst},
};

pub fn optimize_lmc_asm<'a>(
    mut raw_insts: Vec<RawInst<'a>>,
    symbol_table: SymbolTable<'a>,
) -> CompilerResult<'a, (Vec<RawInst<'a>>, SymbolTable<'a>)> {
    let mut i = 0;

    while i < raw_insts.len() {
        let current = raw_insts.get(i).unwrap();

        match current.clone() {
            RawInst {
                label,
                opcode: (_opcode_span, OpCode::Add),
                operand: Some((_operand_span, Operand::Immediate(OperandValue::Value(0)))),
            } if i + 1 < raw_insts.len() => {
                let other = raw_insts.get(i + 1).unwrap();

                match other.clone() {
                    RawInst {
                        label: None,
                        opcode,
                        operand,
                    } => {
                        let new_inst = RawInst {
                            label,
                            opcode,
                            operand,
                        };

                        let mut before = raw_insts[..i].to_vec();
                        let mut after = raw_insts[i + 2..].to_vec();

                        before.push(new_inst);
                        before.append(&mut after);

                        raw_insts = before;
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        i += 1;
    }

    Ok((raw_insts, symbol_table))
}
