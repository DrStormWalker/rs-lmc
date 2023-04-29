use core::fmt;

use regex::Regex;

use crate::{
    error::{CompilerError, CompilerResult},
    span::Span,
};

use std::borrow::Cow;

lazy_static::lazy_static! {
    pub static ref IDENT_RE: Regex = Regex::new(r"^[\p{Alphabetic}_][\p{Alphabetic}\d:_]*$").unwrap();
    pub static ref NUMBER_START_RE: Regex = Regex::new("^[0-9]").unwrap();
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::SemiColon => ";",
                Self::Percent => "%",
                Self::LBracket => "(",
                Self::RBracket => ")",
                Self::LSquare => "[",
                Self::RSquare => "]",
                Self::LCurly => "{",
                Self::RCurly => "}",
                Self::At => "@",
                Self::Hash => "#",
                Self::Comma => ",",
                Self::Colon => ":",
                Self::Equal => "=",
                Self::LineEnd => "end of line",
                Self::Ident => "identifier",
                Self::Literal => "literal",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub(crate) type_: TokenType,
    pub(crate) source: Cow<'a, str>,
    pub(crate) span: Span,
}

pub fn tokenize_lmc_asm<'a>(asm: &'a str) -> CompilerResult<'a, Vec<Token>> {
    let mut iter = asm.char_indices().peekable();

    let mut errors = vec![];
    let mut tokens = vec![];
    let mut line_number = 0;
    let mut line_start = 0;

    while let Some((j, char)) = iter.next() {
        match char {
            '\n' => {
                tokens.push(Token {
                    type_: TokenType::LineEnd,
                    source: Cow::Borrowed(&asm[j..=j]),
                    span: Span::new(j - line_start, j - line_start + 1, line_number),
                });
                line_number += 1;
                line_start = j + 1;
            }
            ';' => {
                let start = j;
                let mut end = j;

                while let Some((j, _)) = iter.next_if(|(_, char)| *char != '\n') {
                    end = j;
                }
            }
            '%' | '(' | ')' | '[' | ']' | '{' | '}' | '@' | '#' | ',' | ':' | '=' => {
                tokens.push(Token {
                    type_: TokenType::try_from_char(char).unwrap(),
                    source: Cow::Borrowed(&asm[j..=j]),
                    span: Span::new(j - line_start, j - line_start + 1, line_number),
                })
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = j;
                let mut end = j;

                while let Some((j, _)) =
                    iter.next_if(|(_, char)| char.is_alphanumeric() || *char == ':' || *char == '_')
                {
                    end = j;
                }

                tokens.push(Token {
                    type_: TokenType::Ident,
                    source: Cow::Borrowed(&asm[start..end + 1]),
                    span: Span::new(start - line_start, end - line_start + 1, line_number),
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
                    source: Cow::Borrowed(&asm[start..end + 1]),
                    span: Span::new(start - line_start, end - line_start + 1, line_number),
                })
            }
            c if c.is_whitespace() => {}
            _ => errors.push(CompilerError::InvalidCharacter(
                asm[j..=j].to_string(),
                Span::new(j - line_start, j - line_start + 1, line_number),
            )),
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(tokens)
    }
}
