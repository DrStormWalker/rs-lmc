use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    iter,
};

use crate::{
    assembler::tokenizer::TokenType,
    error::{CompilerError, CompilerResult},
    instruction::OpCode,
    span::Span,
};

use super::tokenizer::Token;

pub fn macro_assert<'a>(span: Span, result: bool) -> CompilerResult<'a, ()> {
    if !result {
        return Err(vec![CompilerError::UnreachableCondition(span)]);
    }

    Ok(())
}

const STACK_PTR_LABEL: &'static str = "STACK_PTR";
const STACK_LABEL: &'static str = "STACK";
const CALL_STACK_PTR_LABEL: &'static str = "CALL_STACK_PTR";
const CALL_STACK_LABEL: &'static str = "CALL_STACK";

const RET_LABEL: &'static str = "_ret";

pub fn expand_init_macro<'a, 'b>(
    hash: &str,
    tokens: &'b [Token<'a>],
    percent_span: Span,
) -> CompilerResult<'a, Vec<Token<'a>>> {
    let mut stack_size = 16;
    let mut call_stack_size = 16;

    let mut token_iter = tokens.into_iter().peekable();

    if let Some(open) = token_iter.next() {
        macro_assert(open.span, open.type_ == TokenType::LBracket)?;

        let mut depth = 1;

        while let Some(token) = token_iter.next() {
            match token.type_ {
                TokenType::RBracket if depth <= 1 => break,
                TokenType::RBracket => depth -= 1,
                TokenType::LineEnd => {}

                TokenType::Ident => {
                    let mut key = token.source.to_string();
                    while let Some(token) = token_iter.next() {
                        match token.type_ {
                            TokenType::Equal => {
                                let Some(literal) = token_iter.next() else {
                                    return Err(vec![CompilerError::ExpectedToken(
                                        Span::new(token.span.end + 2, token.span.end + 3, token.span.line),
                                        vec!["literal"],
                                    )]);
                                };

                                let Some(comma) = token_iter.peek() else {
                                    return Err(vec![CompilerError::ExpectedToken(
                                        Span::new(literal.span.end + 2, literal.span.end + 3, literal.span.line),
                                        vec!["`,`", "`)`"],
                                    )]);
                                };

                                match comma.type_ {
                                    TokenType::RBracket => {}
                                    TokenType::Comma => {
                                        token_iter.next().unwrap();
                                    }
                                    _ => {
                                        return Err(vec![CompilerError::InvalidToken {
                                            token: comma.source.to_string(),
                                            span: comma.span,
                                            expected: vec!["`,`", "`)`"],
                                        }])
                                    }
                                };

                                match &key[..] {
                                    "stack size" => stack_size = literal.source.parse().unwrap(),
                                    "call stack size" => {
                                        call_stack_size = literal.source.parse().unwrap()
                                    }
                                    _ => todo!("{}", key),
                                }

                                break;
                            }
                            TokenType::Ident => {
                                key += " ";
                                key += &token.source;
                            }
                            TokenType::LineEnd => {}

                            _ => {
                                return Err(vec![CompilerError::InvalidToken {
                                    token: token.source.to_string(),
                                    span: token.span,
                                    expected: vec!["identifier", "`=`"],
                                }])
                            }
                        }
                    }
                }
                _ => {
                    return Err(vec![CompilerError::InvalidToken {
                        token: token.source.to_string(),
                        span: token.span,
                        expected: vec!["identifier", "`)`"],
                    }])
                }
            }
        }
    }

    let mut result = vec![
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(STACK_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Bra.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_init_end", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Dat.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Literal,
            source: Cow::Borrowed(STACK_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Dat.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Literal,
            source: Cow::Borrowed(CALL_STACK_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ];

    result.append(&mut vec![
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(STACK_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Dat.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ]);

    let mut i = 0;

    if stack_size > 0 {
        result.append(
            &mut iter::repeat_with(|| {
                i += 1;
                vec![
                    Token {
                        type_: TokenType::Ident,
                        source: Cow::Owned(format!("_{}_stack_{}", hash, i)),
                        span: percent_span,
                    },
                    Token {
                        type_: TokenType::Ident,
                        source: Cow::Borrowed(OpCode::Dat.as_str()),
                        span: percent_span,
                    },
                    Token {
                        type_: TokenType::LineEnd,
                        source: Cow::Borrowed("\n"),
                        span: percent_span,
                    },
                ]
            })
            .take(stack_size - 1)
            .flatten()
            .collect::<Vec<_>>(),
        );
    }

    result.append(&mut vec![
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Dat.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ]);

    if call_stack_size > 0 {
        let mut i = 0;

        result.append(
            &mut iter::repeat_with(|| {
                i += 1;
                vec![
                    Token {
                        type_: TokenType::Ident,
                        source: Cow::Owned(format!("_{}_call_stack_{}", hash, i)),
                        span: percent_span,
                    },
                    Token {
                        type_: TokenType::Ident,
                        source: Cow::Borrowed(OpCode::Dat.as_str()),
                        span: percent_span,
                    },
                    Token {
                        type_: TokenType::LineEnd,
                        source: Cow::Borrowed("\n"),
                        span: percent_span,
                    },
                ]
            })
            .take(call_stack_size - 1)
            .flatten()
            .collect::<Vec<_>>(),
        );
    }

    result.append(&mut vec![
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(RET_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_tmp", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sub.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Literal,
            source: Cow::Borrowed("1"),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::At,
            source: Cow::Borrowed("@"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_ptr", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_tmp", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Bra.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::At,
            source: Cow::Borrowed("@"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_ptr", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_tmp", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Dat.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_ptr", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Dat.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ]);

    result.append(&mut vec![
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_init_end", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Add.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Literal,
            source: Cow::Borrowed("0"),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ]);

    Ok(result)
}

pub fn expand_start_macro<'a, 'b>(
    tokens: &'b [Token<'a>],
    percent_span: Span,
) -> CompilerResult<'a, Vec<Token<'a>>> {
    if tokens.len() > 0 {
        let token = tokens.first().unwrap();

        Err(vec![CompilerError::UnexpectedTokens(token.span)])
    } else {
        Ok(vec![
            Token {
                type_: TokenType::LineEnd,
                source: Cow::Borrowed("\n"),
                span: percent_span,
            },
            Token {
                type_: TokenType::Ident,
                source: Cow::Borrowed(OpCode::Bra.as_str()),
                span: percent_span,
            },
            Token {
                type_: TokenType::Ident,
                source: Cow::Borrowed("_start"),
                span: percent_span,
            },
            Token {
                type_: TokenType::LineEnd,
                source: Cow::Borrowed("\n"),
                span: percent_span,
            },
        ])
    }
}

pub fn expand_label_macro<'a, 'b>(
    label: &'b Token<'a>,
    tokens: &'b [Token<'a>],
    percent_span: Span,
) -> CompilerResult<'a, Vec<Token<'a>>> {
    if tokens.len() > 0 {
        let token = tokens.first().unwrap();

        Err(vec![CompilerError::UnexpectedTokens(token.span)])
    } else {
        Ok(vec![
            Token {
                type_: TokenType::LineEnd,
                source: Cow::Borrowed("\n"),
                span: percent_span,
            },
            Token {
                type_: TokenType::Ident,
                source: Cow::Owned(label.source[..label.source.len() - 1].to_string()),
                span: Span::new(label.span.start, label.span.end - 1, label.span.line),
            },
            Token {
                type_: TokenType::Ident,
                source: Cow::Borrowed(OpCode::Add.as_str()),
                span: percent_span,
            },
            Token {
                type_: TokenType::Hash,
                source: Cow::Borrowed("#"),
                span: percent_span,
            },
            Token {
                type_: TokenType::Literal,
                source: Cow::Borrowed("0"),
                span: percent_span,
            },
            Token {
                type_: TokenType::LineEnd,
                source: Cow::Borrowed("\n"),
                span: percent_span,
            },
        ])
    }
}

pub fn push_macro<'a, 'b>(hash: &str, label: &'b Token<'a>, percent_span: Span) -> Vec<Token<'a>> {
    vec![
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_tmp", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::At,
            source: Cow::Borrowed("@"),
            span: percent_span,
        },
        label.clone(),
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        label.clone(),
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Add.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Literal,
            source: Cow::Borrowed("1"),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        label.clone(),
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Bra.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_end", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_tmp", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Dat.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_end", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_tmp", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ]
}

pub fn expand_push_macro<'a, 'b>(
    hash: &str,
    tokens: &'b [Token<'a>],
    percent_span: Span,
) -> CompilerResult<'a, Vec<Token<'a>>> {
    let label = if tokens.len() > 0 {
        let open = tokens.get(0).unwrap();
        macro_assert(open.span, open.type_ == TokenType::LBracket)?;

        macro_assert(open.span, tokens.len() > 1)?;
        let token = tokens.get(1).unwrap();

        if token.type_ != TokenType::Ident {
            return Err(vec![CompilerError::InvalidToken {
                token: token.source.to_string(),
                span: token.span,
                expected: vec!["label"],
            }]);
        }

        macro_assert(open.span, tokens.len() > 2)?;
        let close = tokens.get(2).unwrap();

        if close.type_ != TokenType::RBracket {
            return Err(vec![CompilerError::InvalidToken {
                token: close.source.to_string(),
                span: close.span,
                expected: vec!["`)`"],
            }]);
        }

        token.clone()
    } else {
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(STACK_PTR_LABEL),
            span: percent_span,
        }
    };

    //      STA arg1
    //      LDA #_end
    //      STA arg2
    //      BRA push
    // _end ADD #0

    //      STA _tmp
    //      STA @%ptr
    //      LDA %ptr
    //      ADD #1
    //      STA %ptr
    //      BRA _end
    // _tmp DAT
    // _end LDA _tmp

    Ok(push_macro(hash, &label, percent_span))
}

pub fn pop_macro<'a, 'b>(label: &'b Token<'a>, percent_span: Span) -> Vec<Token<'a>> {
    vec![
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        label.clone(),
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sub.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Literal,
            source: Cow::Borrowed("1"),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        label.clone(),
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::At,
            source: Cow::Borrowed("@"),
            span: percent_span,
        },
        label.clone(),
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ]
}

pub fn expand_pop_macro<'a, 'b>(
    tokens: &'b [Token<'a>],
    percent_span: Span,
) -> CompilerResult<'a, Vec<Token<'a>>> {
    let label = if tokens.len() > 0 {
        let open = tokens.get(0).unwrap();
        macro_assert(open.span, open.type_ == TokenType::LBracket)?;

        macro_assert(open.span, tokens.len() > 1)?;
        let token = tokens.get(1).unwrap();

        if token.type_ != TokenType::Ident {
            return Err(vec![CompilerError::InvalidToken {
                token: token.source.to_string(),
                span: token.span,
                expected: vec!["label"],
            }]);
        }

        macro_assert(open.span, tokens.len() > 2)?;
        let close = tokens.get(2).unwrap();

        if close.type_ != TokenType::RBracket {
            return Err(vec![CompilerError::InvalidToken {
                token: close.source.to_string(),
                span: close.span,
                expected: vec!["`)`"],
            }]);
        }

        token.clone()
    } else {
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(STACK_PTR_LABEL),
            span: percent_span,
        }
    };

    //      LDA %ptr
    //      SUB #1
    //      STA %ptr
    //      LDA @%ptr

    Ok(pop_macro(&label, percent_span))
}

pub fn expand_bsub_macro<'a, 'b>(
    hash: &str,
    label_span: Span,
    tokens: &'b [Token<'a>],
    percent_span: Span,
) -> CompilerResult<'a, Vec<Token<'a>>> {
    let Some(open) = tokens.get(0) else {
        return Err(vec![CompilerError::ExpectedToken(
            Span::new(label_span.end + 2, label_span.end + 3, label_span.line),
            vec!["`(`"]
        )]);
    };
    macro_assert(open.span, open.type_ == TokenType::LBracket)?;

    macro_assert(open.span, tokens.len() > 1)?;
    let label = tokens.get(1).unwrap();

    if label.type_ != TokenType::Ident {
        return Err(vec![CompilerError::InvalidToken {
            token: label.source.to_string(),
            span: label.span,
            expected: vec!["label"],
        }]);
    }

    macro_assert(open.span, tokens.len() > 2)?;
    let close = tokens.get(2).unwrap();

    if close.type_ != TokenType::RBracket {
        return Err(vec![CompilerError::InvalidToken {
            token: close.source.to_string(),
            span: close.span,
            expected: vec!["`)`"],
        }]);
    }

    //      LDA #_end
    //      STA @%ptr
    //      LDA #%label
    //      BRA bsub
    // _end ADD #0

    // bsub STA arg
    //      LDA %ptr
    //      ADD #1
    //      STA %ptr
    //      BRA @arg

    //      LDA #_end
    //      STA @%ptr
    //      LDA %ptr
    //      ADD #1
    //      STA %ptr
    //      BRA %label
    // _end ADD #0

    Ok(vec![
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_end", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::At,
            source: Cow::Borrowed("@"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Lda.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Add.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Literal,
            source: Cow::Borrowed("1"),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Sta.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Bra.as_str()),
            span: percent_span,
        },
        label.clone(),
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Owned(format!("_{}_end", hash)),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Add.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Hash,
            source: Cow::Borrowed("#"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Literal,
            source: Cow::Borrowed("0"),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ])
}

pub fn expand_ret_macro<'a, 'b>(
    hash: &str,
    tokens: &'b [Token<'a>],
    percent_span: Span,
) -> CompilerResult<'a, Vec<Token<'a>>> {
    if tokens.len() > 0 {
        let token = tokens.first().unwrap();

        return Err(vec![CompilerError::UnexpectedTokens(token.span)]);
    }

    //      STA _tmp
    //      LDA %ptr
    //      SUB #1
    //      STA %ptr
    //      LDA @%ptr
    //      STA _ptr
    //      LDA _tmo
    //      BRA @_ptr

    Ok(vec![
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Sta.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Owned(format!("_{}_tmp", hash)),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Lda.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Sub.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Hash,
        //         source: Cow::Borrowed("#"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Literal,
        //         source: Cow::Borrowed("1"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Sta.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Lda.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::At,
        //         source: Cow::Borrowed("@"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(CALL_STACK_PTR_LABEL),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Sta.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Owned(format!("_{}_ptr", hash)),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Lda.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Owned(format!("_{}_tmp", hash)),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Bra.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::At,
        //         source: Cow::Borrowed("@"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Owned(format!("_{}_ptr", hash)),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Owned(format!("_{}_tmp", hash)),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Dat.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Owned(format!("_{}_ptr", hash)),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::Ident,
        //         source: Cow::Borrowed(OpCode::Dat.as_str()),
        //         span: percent_span,
        //     },
        //     Token {
        //         type_: TokenType::LineEnd,
        //         source: Cow::Borrowed("\n"),
        //         span: percent_span,
        //     },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(OpCode::Bra.as_str()),
            span: percent_span,
        },
        Token {
            type_: TokenType::Ident,
            source: Cow::Borrowed(RET_LABEL),
            span: percent_span,
        },
        Token {
            type_: TokenType::LineEnd,
            source: Cow::Borrowed("\n"),
            span: percent_span,
        },
    ])
}

pub fn expand_macro<'a, 'b>(tokens: &'b [Token<'a>]) -> CompilerResult<'a, Vec<Token<'a>>> {
    let percent = tokens.first().unwrap();
    macro_assert(percent.span, percent.type_ == TokenType::Percent)?;

    let Some(ident) = tokens.get(1) else {
        return Err(vec![CompilerError::ExpectedToken(
            Span::new(percent.span.end + 1, percent.span.end + 2, percent.span.line),
            vec!["identifier"],
        )]);
    };
    if ident.type_ != TokenType::Ident {
        return Err(vec![CompilerError::InvalidToken {
            token: ident.source.to_string(),
            span: ident.span,
            expected: vec!["identifier"],
        }]);
    }

    let mut hasher = DefaultHasher::new();
    percent.span.hash(&mut hasher);
    let hash = format!("{:x}", hasher.finish());

    match &ident.source[..] {
        "init" => expand_init_macro(&hash, &tokens[2..], percent.span),
        "_start" => expand_start_macro(&tokens[2..], percent.span),
        "push" => expand_push_macro(&hash, &tokens[2..], percent.span),
        "pop" => expand_pop_macro(&tokens[2..], percent.span),
        "bsub" => expand_bsub_macro(&hash, ident.span, &tokens[2..], percent.span),
        "ret" => expand_ret_macro(&hash, &tokens[2..], percent.span),
        label if label.ends_with(':') => expand_label_macro(ident, &tokens[2..], percent.span),
        _ => Ok(vec![]),
    }
}

pub fn expand_lmc_macros<'a>(mut tokens: Vec<Token<'a>>) -> CompilerResult<Vec<Token<'a>>> {
    let mut i = 0;

    let mut macro_start = 0;
    let mut macro_depth = 0;

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
                    if macro_depth > 0 {
                        macro_depth += 1;
                    }
                }
                TokenType::RBracket => {
                    if macro_depth > 0 {
                        macro_depth -= 1;

                        exit_macro = macro_depth <= 1;
                    }
                }
                _ => {}
            }
        }

        i += 1;

        if exit_macro {
            let mut expanded = expand_macro(&tokens[macro_start..i])?;

            let mut after = tokens[i..].to_vec();
            let mut before = tokens[..macro_start].to_vec();

            before.append(&mut expanded);
            before.append(&mut after);

            tokens = before;
            macro_depth = 0;
        }
    }

    Ok(tokens)
}
