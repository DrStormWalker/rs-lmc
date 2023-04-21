use regex::Regex;

use crate::{
    error::{CompilerError, CompilerResult},
    span::Span,
};

lazy_static::lazy_static! {
    pub static ref IDENT_RE: Regex = Regex::new(r"^\p{Alphabetic}[\p{Alphabetic}\d:]*$").unwrap();
    pub static ref NUMBER_START_RE: Regex = Regex::new("^[0-9]").unwrap();
}

#[derive(Copy, Clone, Debug)]
pub enum TokenType {
    SemiColon,
    Percent,
    LBracket,
    RBracket,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    At,
    Hash,
    Comma,
    Colon,
    Equal,

    LineEnd,

    Ident,
    Literal,
}
impl TokenType {
    pub fn try_from_char(char: char) -> Option<Self> {
        match char {
            ';' => Some(Self::SemiColon),
            '%' => Some(Self::Percent),
            '(' => Some(Self::LBracket),
            ')' => Some(Self::RBracket),
            '[' => Some(Self::LSquare),
            ']' => Some(Self::RSquare),
            '{' => Some(Self::LCurly),
            '}' => Some(Self::RCurly),
            '@' => Some(Self::At),
            '#' => Some(Self::Hash),
            ',' => Some(Self::Comma),
            ':' => Some(Self::Colon),
            '=' => Some(Self::Equal),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    pub(crate) type_: TokenType,
    pub(crate) source: &'a str,
    pub(crate) span: Span,
}

pub fn tokenize_lmc_asm<'a>(asm: &'a str) -> CompilerResult<'a, Vec<Token>> {
    let mut iter = asm.char_indices().peekable();

    let mut errors = vec![];
    let mut tokens = vec![];
    let mut line_number = 0;

    while let Some((j, char)) = iter.next() {
        match char {
            '\n' => {
                tokens.push(Token {
                    type_: TokenType::LineEnd,
                    source: &asm[j..=j],
                    span: Span::new(j, j + 1, line_number),
                });
                line_number += 1;
            }
            ';' | '%' | '(' | ')' | '[' | ']' | '{' | '}' | '@' | '#' | ',' | ':' | '=' => tokens
                .push(Token {
                    type_: TokenType::try_from_char(char).unwrap(),
                    source: &asm[j..=j],
                    span: Span::new(j, j + 1, line_number),
                }),
            c if c.is_alphabetic() => {
                let start = j;
                let mut end = j;

                while let Some((j, _)) =
                    iter.next_if(|(_, char)| char.is_alphanumeric() || *char == ':' || *char == '_')
                {
                    end = j;
                }

                tokens.push(Token {
                    type_: TokenType::Ident,
                    source: &asm[start..end + 1],
                    span: Span::new(start, end + 1, line_number),
                })
            }
            c if c.is_ascii_digit() => {
                let start = j;
                let mut end = j;

                while let Some((j, _)) = iter.next_if(|(_, char)| char.is_ascii_digit()) {
                    end = j;
                }

                tokens.push(Token {
                    type_: TokenType::Literal,
                    source: &asm[start..end + 1],
                    span: Span::new(start, end + 1, line_number),
                })
            }
            c if c.is_whitespace() => {}
            _ => errors.push(CompilerError::InvalidCharacter(
                &asm[j..=j],
                Span::new(j, j + 1, line_number),
            )),
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(tokens)
    }
}
