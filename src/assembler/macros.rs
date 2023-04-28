use crate::{assembler::tokenizer::TokenType, error::CompilerResult};

use super::tokenizer::Token;

pub fn expand_macro<'a>(mut tokens: Vec<Token<'a>>) -> CompilerResult<Vec<Token<'a>>> {
    Ok(vec![])
}

pub fn expand_lmc_macros<'a>(mut tokens: Vec<Token<'a>>) -> CompilerResult<Vec<Token<'a>>> {
    let mut i = 0;

    let mut macro_start = 0;
    let mut macro_depth = 0;

    let mut current_macro = vec![];

    while i < tokens.len() {
        let mut exit_macro = false;

        {
            let current = tokens.get(i).unwrap();

            match current.type_ {
                TokenType::Percent => {
                    macro_start = i;
                    macro_depth = 1;
                }
                TokenType::Ident => {
                    if macro_depth == 1 {
                        macro_depth = tokens.get(i + 1).map_or(0, |t| {
                            if t.type_ == TokenType::LBracket {
                                1
                            } else {
                                0
                            }
                        });

                        exit_macro = macro_depth < 1;
                    }
                }
                TokenType::LBracket => {
                    if macro_depth > 1 {
                        macro_depth += 1;
                    }
                }
                TokenType::RBracket => {
                    if macro_depth > 1 {
                        macro_depth -= 1;

                        exit_macro = macro_depth == 0;
                    }
                }
                _ => {}
            }

            if macro_depth > 1 || exit_macro {
                current_macro.push(*current);
            }
        }

        i += 1;

        if exit_macro {
            let mut after = tokens[i..].to_vec();
            let mut before = tokens[..macro_start].to_vec();

            let mut expanded = expand_macro(tokens)?;

            current_macro = vec![];
            before.append(&mut expanded);
            before.append(&mut after);
            tokens = before;
        }
    }

    Ok(tokens)
}
